use std::collections::{HashMap, HashSet};

use crate::{
    ActionTemplateExecutor,
    action::ActionTemplate,
    actor::{self, actor_factory::ActorFactory, actor_state_extractor::ActorStateExtractor},
    create_executor,
    global_state::GlobalState,
    transition::Transition,
};

pub struct TransitionComputer {
    actions: HashSet<ActionTemplate>,
    action_template_executor: Box<dyn ActionTemplateExecutor>,
}

impl TransitionComputer {
    pub fn new(
        actions: HashSet<ActionTemplate>,
        actor_factories: HashMap<actor::Id, Box<dyn ActorFactory>>,
        actor_state_extractors: HashMap<actor::Id, Box<dyn ActorStateExtractor>>,
    ) -> Self {
        Self {
            actions,
            action_template_executor: create_executor(actor_factories, actor_state_extractors),
        }
    }

    pub async fn compute(&self, from: GlobalState) -> HashSet<Transition> {
        let mut transitions = HashSet::new();

        for action_template in self.actions.iter() {
            let result = self
                .action_template_executor
                .execute(action_template.clone(), from.clone())
                .await;
            transitions.insert(Transition {
                from: from.clone(),
                to: result.global_states.clone(),
                action_template: action_template.clone(),
                action_result: result.action_result,
            });
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
        action::{
            ActionResult, ActionTemplate, ActionType, ExecutionResult, MockActionTemplateExecutor,
        },
        actor::{self, ActorBase, actor_state::ActorState, local_state::LocalState},
        global_state::GlobalState,
        test_utils::test_actors::{TestActor1State, TestActor2State},
        transition::Transition,
        transition_computer::TransitionComputer,
    };

    static ACTOR_1_ID: LazyLock<actor::Id> = LazyLock::new(|| actor::Id("actor_1".to_string()));
    static ACTOR_2_ID: LazyLock<actor::Id> = LazyLock::new(|| actor::Id("actor_2".to_string()));

    #[tokio::test]
    async fn works() {
        let actions: HashSet<ActionTemplate> = create_actions();
        let global_state_0 = global_state(0, 0);
        let global_state_1 = global_state(0, 1);
        let global_state_2 = global_state(1, 0);
        let mut executor = MockActionTemplateExecutor::new();
        prepare_executor(
            &mut executor,
            action_a(),
            global_state_0.clone(),
            global_state_1.clone(),
        );
        prepare_executor(
            &mut executor,
            action_b(),
            global_state_0.clone(),
            global_state_2.clone(),
        );

        let state_machine_driver = TransitionComputer {
            actions,
            action_template_executor: Box::new(executor),
        };

        assert_eq!(
            state_machine_driver.compute(global_state_0.clone()).await,
            HashSet::from([
                Transition {
                    from: global_state_0.clone(),
                    to: global_state_1,
                    action_template: action_a(),
                    action_result: ActionResult(None)
                },
                Transition {
                    from: global_state_0.clone(),
                    to: global_state_2,
                    action_template: action_b(),
                    action_result: ActionResult(None)
                }
            ])
        );
    }

    fn prepare_executor(
        executor: &mut MockActionTemplateExecutor,
        action: ActionTemplate,
        from: GlobalState,
        to: GlobalState,
    ) {
        executor
            .expect_execute()
            .with(eq(action), eq(from))
            .once()
            .return_once(|_, _| ExecutionResult {
                action_result: ActionResult(None),
                global_states: to,
            });
    }

    fn global_state(actor_1_state_value: u8, actor_2_state_value: u8) -> GlobalState {
        GlobalState::new(BTreeMap::from([
            (
                ACTOR_1_ID.clone(),
                LocalState {
                    actor_state: actor_1_state(actor_1_state_value),
                },
            ),
            (
                ACTOR_2_ID.clone(),
                LocalState {
                    actor_state: actor_2_state(actor_2_state_value),
                },
            ),
        ]))
    }

    fn actor_1_state(value: u8) -> Arc<dyn ActorState> {
        Arc::new(TestActor1State { value })
    }

    fn actor_2_state(value: u8) -> Arc<dyn ActorState> {
        Arc::new(TestActor2State { value })
    }

    fn create_actions() -> HashSet<ActionTemplate> {
        HashSet::from([action_a(), action_b()])
    }

    fn action_a() -> ActionTemplate {
        ActionTemplate {
            performer_id: ACTOR_1_ID.clone(),
            label: "action-A",
            action_type: ActionType::Intransitive(Arc::new(|actor| {
                Box::pin(proxy_for_intransitive_action(actor))
            })),
        }
    }

    fn action_b() -> ActionTemplate {
        ActionTemplate {
            performer_id: ACTOR_1_ID.clone(),
            label: "action-B",
            action_type: ActionType::Transitive {
                receiver_id: ACTOR_2_ID.clone(),
                action: Arc::new(|performer, receiver| {
                    Box::pin(proxy_for_transitive_action(performer, receiver))
                }),
            },
        }
    }

    async fn proxy_for_intransitive_action(
        _: Arc<dyn ActorBase>,
    ) -> Result<(), Box<dyn Error + Send>> {
        Ok(())
    }

    async fn proxy_for_transitive_action(
        _: Arc<dyn ActorBase>,
        _: Arc<dyn ActorBase>,
    ) -> Result<(), Box<dyn Error + Send>> {
        Ok(())
    }
}
