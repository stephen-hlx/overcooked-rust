use std::collections::{HashSet, VecDeque};

use crate::{global_state::GlobalState, transition::Transition};

mod simple_transition_computer;

#[mockall::automock]
#[async_trait::async_trait]
pub trait TransitionComputer {
    async fn compute(&self, from: GlobalState) -> HashSet<Transition>;
}

pub struct StateMachineDriver {
    transition_computer: Box<dyn TransitionComputer>,
}

impl StateMachineDriver {
    pub async fn run(&self, initial_state: GlobalState) -> HashSet<Transition> {
        let mut transitions = HashSet::new();

        let mut queue = VecDeque::new();
        queue.push_back(initial_state);

        let mut visited = HashSet::new();

        while let Some(curr) = queue.pop_front() {
            if visited.contains(&curr) {
                continue;
            } else {
                visited.insert(curr.clone());
            }

            let out_going_transitions = self.transition_computer.compute(curr).await;
            transitions.extend(out_going_transitions.clone());

            queue.extend(
                out_going_transitions
                    .into_iter()
                    .map(|transition| transition.to)
                    .collect::<HashSet<_>>(),
            );
        }

        transitions
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, HashSet},
        error::Error,
        sync::{Arc, LazyLock},
    };

    use mockall::predicate::eq;

    use crate::{
        action::{ActionResult, ActionTemplate, ActionType},
        actor::{self, ActorBase, actor_state::ActorState, local_state::LocalState},
        global_state::GlobalState,
        state_machine_driver::{MockTransitionComputer, StateMachineDriver},
        test_utils::test_actors::TestActor1State,
        transition::Transition,
    };

    static ACTOR_ID: LazyLock<actor::Id> = LazyLock::new(|| actor::Id("actor_1".to_string()));

    static ACTION_A: &str = "action_a";
    static ACTION_B: &str = "action_b";
    static ACTION_C: &str = "action_c";
    static ACTION_D: &str = "action_d";
    static ACTION_E: &str = "action_e";

    /// The state machine looks like this:
    ///      ┌──────┐
    ///      │ GS_0 │
    ///      └──┬───┘
    ///     ┌───┴────┐
    ///    a│       b│  ┌──┐
    /// ┌───▼──┐  ┌──▼──┴┐ │b
    /// │ GS_1 │  │ GS_2 ◄─┘
    /// └───┬──┘  └──┬───┘
    ///    c│       c│e ┌──┐
    /// ┌───▼──┐ d┌──▼──┴┐ │d
    /// │ GS_3 ├──► GS_4 ◄─┘
    /// └──────┘  └──────┘
    #[tokio::test]
    async fn works() {
        let mut transition_computer = MockTransitionComputer::new();

        let global_state_0 = global_state(0);
        let global_state_1 = global_state(1);
        let global_state_2 = global_state(2);
        let global_state_3 = global_state(3);
        let global_state_4 = global_state(4);

        //      ┌──────┐
        //      │ GS_0 │
        //      └──┬───┘
        //     ┌───┴────┐
        //    a│       b│
        // ┌───▼──┐  ┌──▼───┐
        // │ GS_1 │  │ GS_2 │
        // └──────┘  └──────┘
        let transition_0_a_1 = transition(global_state_0.clone(), global_state_1.clone(), ACTION_A);
        let transition_0_b_2 = transition(global_state_0.clone(), global_state_2.clone(), ACTION_B);

        //       ┌──┐
        // ┌─────┴┐ │b
        // │ GS_2 ◄─┘
        // └───┬──┘
        //    c│e
        // ┌───▼──┐
        // │ GS_4 │
        // └──────┘
        let transition_2_b_2 = transition(global_state_2.clone(), global_state_2.clone(), ACTION_B);
        let transition_2_c_4 = transition(global_state_2.clone(), global_state_4.clone(), ACTION_C);
        let transition_2_e_4 = transition(global_state_2.clone(), global_state_4.clone(), ACTION_E);

        // ┌──────┐
        // │ GS_1 │
        // └───┬──┘
        //    c│
        // ┌───▼──┐
        // │ GS_3 │
        // └──────┘
        let transition_1_c_3 = transition(global_state_1.clone(), global_state_3.clone(), ACTION_C);

        //       ┌──┐
        // ┌─────┴┐ │b
        // │ GS_2 ◄─┘
        // └───┬──┘
        //    c│e
        // ┌───▼──┐
        // │ GS_4 │
        // └──────┘
        let transition_2_b_2 = transition(global_state_2.clone(), global_state_2.clone(), ACTION_B);
        let transition_2_c_4 = transition(global_state_2.clone(), global_state_4.clone(), ACTION_C);
        let transition_2_e_4 = transition(global_state_2.clone(), global_state_4.clone(), ACTION_E);

        // ┌──────┐ d┌──────┐
        // │ GS_3 ├──► GS_4 │
        // └──────┘  └──────┘
        let transition_3_d_4 = transition(global_state_3.clone(), global_state_4.clone(), ACTION_D);

        //       ┌──┐
        // ┌─────┴┐ │d
        // │ GS_4 ◄─┘
        // └──────┘
        let transition_4_d_4 = transition(global_state_4.clone(), global_state_4.clone(), ACTION_D);

        let transition_0_a_1_clone = transition_0_a_1.clone();
        let transition_0_b_2_clone = transition_0_b_2.clone();
        let transition_2_b_2_clone = transition_2_b_2.clone();
        let transition_1_c_3_clone = transition_1_c_3.clone();
        let transition_2_c_4_clone = transition_2_c_4.clone();
        let transition_2_e_4_clone = transition_2_e_4.clone();
        let transition_3_d_4_clone = transition_3_d_4.clone();
        let transition_4_d_4_clone = transition_4_d_4.clone();

        transition_computer
            .expect_compute()
            .with(eq(global_state_0.clone()))
            .once()
            .return_once(|_| HashSet::from([transition_0_a_1, transition_0_b_2]));
        transition_computer
            .expect_compute()
            .with(eq(global_state_2.clone()))
            .once()
            .return_once(|_| HashSet::from([transition_2_b_2, transition_2_c_4, transition_2_e_4]));
        transition_computer
            .expect_compute()
            .with(eq(global_state_1))
            .once()
            .return_once(|_| HashSet::from([transition_1_c_3]));
        transition_computer
            .expect_compute()
            .with(eq(global_state_3))
            .once()
            .return_once(|_| HashSet::from([transition_3_d_4]));
        transition_computer
            .expect_compute()
            .with(eq(global_state_4))
            .once()
            .return_once(|_| HashSet::from([transition_4_d_4]));

        let state_machine_driver = StateMachineDriver {
            transition_computer: Box::new(transition_computer),
        };

        assert_eq!(
            state_machine_driver.run(global_state_0).await,
            HashSet::from([
                transition_0_a_1_clone,
                transition_0_b_2_clone,
                transition_1_c_3_clone,
                transition_2_b_2_clone,
                transition_2_c_4_clone,
                transition_2_e_4_clone,
                transition_3_d_4_clone,
                transition_4_d_4_clone,
            ])
        );
    }

    // TODO: group these utils under the module of GlobalState
    fn global_state(actor_state_value: u8) -> GlobalState {
        GlobalState::new(BTreeMap::from([(
            ACTOR_ID.clone(),
            LocalState {
                actor_state: actor_state(actor_state_value),
            },
        )]))
    }

    fn transition(from: GlobalState, to: GlobalState, action_label: &str) -> Transition {
        Transition {
            from,
            to,
            action_template: action(action_label),
            action_result: ActionResult(None),
        }
    }

    fn actor_state(value: u8) -> Arc<dyn ActorState> {
        Arc::new(TestActor1State { value })
    }

    fn action(label: &str) -> ActionTemplate {
        ActionTemplate {
            performer_id: ACTOR_ID.clone(),
            label: label.to_string(),
            action_type: ActionType::Intransitive(Arc::new(|actor| {
                Box::pin(proxy_for_intransitive_action(actor))
            })),
        }
    }

    /// todo: move this to a util module
    async fn proxy_for_intransitive_action(
        _: Arc<dyn ActorBase>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}
