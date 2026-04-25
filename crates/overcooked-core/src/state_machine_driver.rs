use std::collections::{HashSet, VecDeque};

use crate::{
    global_state::GlobalState, state_machine_execution_result::StateMachineExecutionResult,
    transition::Transition,
};

mod simple_transition_computer;

/// Given a [`GlobalState`], computes the outgoing [`Transition`]s.
#[mockall::automock]
#[async_trait::async_trait]
pub trait TransitionComputer {
    async fn compute(&self, from: GlobalState) -> HashSet<Transition>;
}

/// An invariant is a property of a system that should always be upheld.
///
/// The invariant is defined using the state of each component of a system.
///
/// This trait offers an opportunity for the client to specify the invariant in the form of
/// [`GlobalState`] so that the [`StateMachineDriver`] can use it to verify each of the states it
/// discovered.
#[mockall::automock]
pub trait InvariantVerifier {
    fn verify(&self, global_state: &GlobalState) -> Result<(), Box<dyn std::error::Error>>;
}

/// The component that drives the execution of a state machine.
///
/// Given a [`GlobalState`], the driver uses [`TransitionComputer`] to compute the outgoing
/// [`Transition`]s. For each of the not-yet-visited successor states, the driver takes them as new
/// [`GlobalState`]s and computes the outgoing [`Transition`]s, recursively. For each newly
/// discovered [`GlobalState`], the driver will also verify whether the invariant is upheld.
pub struct StateMachineDriver {
    transition_computer: Box<dyn TransitionComputer>,
    invariant_verifier: Box<dyn InvariantVerifier>,
}

impl StateMachineDriver {
    pub async fn run(&self, initial_state: GlobalState) -> StateMachineExecutionResult {
        let mut transitions = HashSet::new();
        let mut invariant_violations = HashSet::new();

        let mut queue = VecDeque::new();
        queue.push_back(initial_state);

        let mut visited = HashSet::new();

        while let Some(curr) = queue.pop_front() {
            if visited.contains(&curr) {
                continue;
            } else {
                visited.insert(curr.clone());
            }

            if self.invariant_verifier.verify(&curr).is_err() {
                invariant_violations.insert(curr.clone());
                continue;
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

        StateMachineExecutionResult {
            transitions,
            invariant_violations,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, HashMap, HashSet},
        error::Error,
        sync::{Arc, LazyLock},
    };

    use mockall::predicate::eq;

    use crate::{
        action::{ActionResult, ActionTemplate, ActionType},
        actor::{self, ActorBase, actor_state::ActorState, local_state::LocalState},
        global_state::GlobalState,
        state_machine_driver::{MockInvariantVerifier, MockTransitionComputer, StateMachineDriver},
        state_machine_execution_result::StateMachineExecutionResult,
        test_utils::test_actors::TestActor1State,
        transition::Transition,
    };

    static ACTOR_ID: LazyLock<actor::Id> = LazyLock::new(|| actor::Id("actor_1".to_string()));

    static ACTION_A: &str = "action_a";
    static ACTION_B: &str = "action_b";
    static ACTION_C: &str = "action_c";
    static ACTION_D: &str = "action_d";
    static ACTION_E: &str = "action_e";
    static ACTION_F: &str = "action_f";

    /// The state machine looks like this:
    ///                ┌──────┐
    ///                │ GS_0 │
    ///                └──┬───┘
    ///               ┌───┴────┐
    ///              a│       b│  ┌──┐
    /// ┌──────┐ f┌───▼──┐  ┌──▼──┴┐ │b
    /// │ GS_5 ◄──┤ GS_1 │  │ GS_2 ◄─┘
    /// └──────┘  └───┬──┘  └──┬───┘
    ///              c│       c│e ┌──┐
    ///           ┌───▼──┐ d┌──▼──┴┐ │d
    ///           │ GS_3 ├──► GS_4 ◄─┘
    ///           └──────┘  └──────┘
    #[tokio::test]
    async fn works() {
        let mut transition_computer = MockTransitionComputer::new();

        let global_state_0 = global_state(0);
        let global_state_1 = global_state(1);
        let global_state_2 = global_state(2);
        let global_state_3 = global_state(3);
        let global_state_4 = global_state(4);
        let global_state_5 = global_state(5);

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

        // ┌──────┐ f┌──────┐
        // │ GS_5 ◄──┤ GS_1 │
        // └──────┘  └───┬──┘
        //              c│
        //           ┌───▼──┐
        //           │ GS_3 │
        //           └──────┘
        let transition_1_c_3 = transition(global_state_1.clone(), global_state_3.clone(), ACTION_C);
        let transition_1_f_5 = transition(global_state_1.clone(), global_state_5.clone(), ACTION_F);

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

        prepare_mock_transition_computer(
            &mut transition_computer,
            HashMap::from([
                (
                    global_state_0.clone(),
                    HashSet::from([transition_0_a_1.clone(), transition_0_b_2.clone()]),
                ),
                (
                    global_state_1.clone(),
                    HashSet::from([transition_1_c_3.clone(), transition_1_f_5.clone()]),
                ),
                (
                    global_state_2.clone(),
                    HashSet::from([
                        transition_2_b_2.clone(),
                        transition_2_c_4.clone(),
                        transition_2_e_4.clone(),
                    ]),
                ),
                (
                    global_state_3.clone(),
                    HashSet::from([transition_3_d_4.clone()]),
                ),
                (
                    global_state_4.clone(),
                    HashSet::from([transition_4_d_4.clone()]),
                ),
            ]),
        );

        let mut invariant_verifier = MockInvariantVerifier::new();
        prepare_mock_verifier(
            &mut invariant_verifier,
            HashMap::from([
                (
                    Err(MockInvariantVerifierError),
                    vec![global_state_5.clone()],
                ),
                (
                    Ok(()),
                    vec![
                        global_state_0.clone(),
                        global_state_1,
                        global_state_2,
                        global_state_3,
                        global_state_4,
                    ],
                ),
            ]),
        );

        let state_machine_driver = StateMachineDriver {
            transition_computer: Box::new(transition_computer),
            invariant_verifier: Box::new(invariant_verifier),
        };

        assert_eq!(
            state_machine_driver.run(global_state_0).await,
            StateMachineExecutionResult {
                transitions: HashSet::from([
                    transition_0_a_1,
                    transition_0_b_2,
                    transition_1_c_3,
                    transition_1_f_5,
                    transition_2_b_2,
                    transition_2_c_4,
                    transition_2_e_4,
                    transition_3_d_4,
                    transition_4_d_4,
                ]),
                invariant_violations: HashSet::from([global_state_5])
            }
        );
    }

    fn prepare_mock_transition_computer(
        transition_computer: &mut MockTransitionComputer,
        from_tos: HashMap<GlobalState, HashSet<Transition>>,
    ) {
        for (from, tos) in from_tos {
            transition_computer
                .expect_compute()
                .with(eq(from))
                .once()
                .return_once(|_| tos);
        }
    }
    fn prepare_mock_verifier(
        invariant_verifier: &mut MockInvariantVerifier,
        results: HashMap<Result<(), MockInvariantVerifierError>, Vec<GlobalState>>,
    ) {
        for (result, global_states) in results {
            for global_state in global_states {
                let result = result.clone();
                invariant_verifier
                    .expect_verify()
                    .with(eq(global_state))
                    .once()
                    .return_once(|_| Ok(result.map_err(|err| Box::new(err))?));
            }
        }
    }

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

    async fn proxy_for_intransitive_action(
        _: Arc<dyn ActorBase>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }

    #[derive(Debug, PartialEq, Eq, Hash, Clone, thiserror::Error)]
    #[error("")]
    pub struct MockInvariantVerifierError;
}
