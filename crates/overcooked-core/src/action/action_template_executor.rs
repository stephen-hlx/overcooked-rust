use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use crate::{
    action::{
        ActionResult, ActionTemplate, ActionTemplateExecutor, ExecutionResult, IntransitiveAction,
        TransitiveAction, action_template_executor::action_executor::SimpleActionExecutor,
    },
    actor::{
        self, ActorBase, actor_factory::ActorFactory, actor_state_extractor::ActorStateExtractor,
        local_state::LocalState,
    },
    global_state::LocalStates,
};

use super::ActionType;

mod action;
mod action_executor;

#[mockall::automock]
#[async_trait::async_trait]
trait ActionExecutor: Sync {
    async fn execute(&self, action: Action) -> ActionResult;
}

enum Action {
    Intransitive {
        performer: Arc<dyn ActorBase>,
        action: IntransitiveAction,
    },
    Transitive {
        performer: Arc<dyn ActorBase>,
        receiver: Arc<dyn ActorBase>,
        action: TransitiveAction,
    },
}

pub(super) struct SimpleActionTemplateExecutor<AE = SimpleActionExecutor>
where
    AE: ActionExecutor,
{
    pub action_executor: AE,
    pub actor_state_extractors: HashMap<actor::Id, Box<dyn ActorStateExtractor>>,
    pub actor_factories: HashMap<actor::Id, Box<dyn ActorFactory>>,
}

#[async_trait::async_trait]
impl<AE> ActionTemplateExecutor for SimpleActionTemplateExecutor<AE>
where
    AE: ActionExecutor,
{
    async fn execute(
        &self,
        template: ActionTemplate,
        local_states: LocalStates,
    ) -> ExecutionResult {
        let performer = self
            .actor_factories
            .get(&template.actor_performer_id)
            .unwrap()
            .restore_from_state(
                local_states
                    .0
                    .get(&template.actor_performer_id)
                    .unwrap()
                    .actor_state
                    .clone(),
            )
            .await;

        let action_result = match (match template.action_type {
            ActionType::Intransitive(action) => action(performer.clone()),
            ActionType::Transitive {
                receiver_id,
                transitive_action,
            } => {
                let receiver = self
                    .actor_factories
                    .get(&receiver_id)
                    .unwrap()
                    .restore_from_state(
                        local_states
                            .0
                            .get(&receiver_id)
                            .unwrap()
                            .actor_state
                            .clone(),
                    )
                    .await;
                transitive_action(performer.clone(), receiver)
            }
        }
        .await)
        {
            Ok(_) => ActionResult(None),
            Err(err) => ActionResult(Some(err)),
        };

        let mut updated_local_states = local_states.clone();
        updated_local_states.0.insert(
            template.actor_performer_id.clone(),
            LocalState {
                actor_state: self
                    .actor_state_extractors
                    .get(&template.actor_performer_id)
                    .unwrap()
                    .extract(performer)
                    .await,
            },
        );

        ExecutionResult {
            action_result,
            local_states: updated_local_states,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, HashMap},
        error::Error,
        sync::{Arc, LazyLock},
    };

    use mockall::predicate::eq;

    use crate::{
        action::{
            ActionResult, ActionTemplate, ActionTemplateExecutor, ActionType, ExecutionResult,
            action_template_executor::{Action, MockActionExecutor, SimpleActionTemplateExecutor},
        },
        actor::{
            self, ActorBase,
            actor_factory::{ActorFactory, MockActorFactory},
            actor_state::ActorState,
            actor_state_extractor::{ActorStateExtractor, MockActorStateExtractor},
            local_state::LocalState,
        },
        global_state::LocalStates,
        test_utils::test_actors::{TestActor1, TestActor1State},
    };

    static ACTOR_1_ID: LazyLock<actor::Id> = LazyLock::new(|| actor::Id("actor_1".to_string()));
    static ACTOR_2_ID: LazyLock<actor::Id> = LazyLock::new(|| actor::Id("actor_2".to_string()));

    #[tokio::test]
    async fn intransitive_action_template_execution_works() {
        let actor: Arc<dyn ActorBase> = Arc::new(TestActor1::new(10));
        let actor_state_original: Arc<dyn ActorState> = Arc::new(TestActor1State { value: 10 });
        let actor_state_updated: Arc<dyn ActorState> = Arc::new(TestActor1State { value: 10 });
        let local_states = LocalStates(BTreeMap::from([(
            ACTOR_1_ID.clone(),
            LocalState {
                actor_state: actor_state_original.clone(),
            },
        )]));

        let mut actor_1_factor = MockActorFactory::new();
        let mut action_executor = MockActionExecutor::new();
        let mut actor_1_state_extractor = MockActorStateExtractor::new();

        let restored_actor = actor.clone();
        actor_1_factor
            .expect_restore_from_state()
            .withf(|actor_state| state_having_value(actor_state, 10))
            .once()
            .return_once(|_| restored_actor);

        action_executor
            .expect_execute()
            .with(eq(Action::Intransitive {
                performer: actor,
                action: Box::new(|actor| Box::pin(proxy_for_intransitive_action(actor))),
            }))
            .once()
            .return_once(|_| ActionResult(None));

        let actor_state_updated_clone = actor_state_updated.clone();
        actor_1_state_extractor
            .expect_extract()
            .withf(|actor| actor_having_value(actor, 10))
            .once()
            .return_once(|_| actor_state_updated_clone);

        let executor = SimpleActionTemplateExecutor {
            action_executor,
            actor_state_extractors: HashMap::from([(
                ACTOR_1_ID.clone(),
                Box::new(actor_1_state_extractor) as Box<dyn ActorStateExtractor>,
            )]),
            actor_factories: HashMap::from([(
                ACTOR_1_ID.clone(),
                Box::new(actor_1_factor) as Box<dyn ActorFactory>,
            )]),
        };

        let execution_result = executor
            .execute(
                ActionTemplate {
                    actor_performer_id: ACTOR_1_ID.clone(),
                    label: "some_intransitive_action",
                    action_type: ActionType::Intransitive(Box::new(|actor| {
                        Box::pin(proxy_for_intransitive_action(actor))
                    })),
                },
                local_states,
            )
            .await;

        assert!(execution_result.action_result.0.is_none());
        assert_eq!(
            execution_result.local_states,
            LocalStates(BTreeMap::from([(
                ACTOR_1_ID.clone(),
                LocalState {
                    actor_state: actor_state_updated,
                },
            )]))
        );
    }

    fn state_having_value(actor_state: &Arc<dyn ActorState>, expected_value: u8) -> bool {
        ActorState::as_any(actor_state.as_ref())
            .downcast_ref::<TestActor1State>()
            .unwrap()
            .value
            == expected_value
    }

    fn actor_having_value(actor: &Arc<dyn ActorBase>, expected_value: u8) -> bool {
        ActorBase::as_any(actor.as_ref())
            .downcast_ref::<TestActor1>()
            .unwrap()
            .get_value()
            == expected_value
    }

    async fn proxy_for_intransitive_action(
        _: Arc<dyn ActorBase>,
    ) -> Result<(), Box<dyn Error + Send>> {
        Ok(())
    }
}
