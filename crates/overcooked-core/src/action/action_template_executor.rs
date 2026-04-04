use std::{collections::HashMap, sync::Arc};

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
pub(super) trait ActionExecutor: Sync {
    async fn execute(&self, action: Action) -> ActionResult;
}

pub(super) enum Action {
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

pub(super) struct SimpleActionTemplateExecutor<AE>
where
    AE: ActionExecutor,
{
    pub action_executor: AE,
    pub actor_factories: HashMap<actor::Id, Box<dyn ActorFactory>>,
    pub actor_state_extractors: HashMap<actor::Id, Box<dyn ActorStateExtractor>>,
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
            .restore_actor(&template.performer_id, &local_states)
            .await;

        let mut updated_local_states = local_states.clone();
        let action_result = match template.action_type {
            ActionType::Intransitive(action) => {
                let action_result = self
                    .action_executor
                    .execute(Action::Intransitive {
                        performer: performer.clone(),
                        action,
                    })
                    .await;

                action_result
            }
            ActionType::Transitive {
                receiver_id,
                action,
            } => {
                let receiver = self.restore_actor(&receiver_id, &local_states).await;
                let action_result = self
                    .action_executor
                    .execute(Action::Transitive {
                        performer: performer.clone(),
                        receiver: receiver.clone(),
                        action,
                    })
                    .await;

                updated_local_states.0.insert(
                    receiver_id.clone(),
                    self.extract_state(&receiver_id, receiver).await,
                );
                action_result
            }
        };

        updated_local_states.0.insert(
            template.performer_id.clone(),
            self.extract_state(&template.performer_id, performer).await,
        );

        ExecutionResult {
            action_result,
            local_states: updated_local_states,
        }
    }
}

impl<AE> SimpleActionTemplateExecutor<AE>
where
    AE: ActionExecutor,
{
    async fn restore_actor(
        &self,
        actor_id: &actor::Id,
        local_states: &LocalStates,
    ) -> Arc<dyn ActorBase> {
        self.actor_factories
            .get(actor_id)
            .unwrap()
            .restore_from_state(local_states.0.get(actor_id).unwrap().actor_state.clone())
            .await
    }

    async fn extract_state(&self, actor_id: &actor::Id, actor: Arc<dyn ActorBase>) -> LocalState {
        LocalState {
            actor_state: self
                .actor_state_extractors
                .get(actor_id)
                .unwrap()
                .extract(actor)
                .await,
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
        test_utils::test_actors::{TestActor1, TestActor1State, TestActor2, TestActor2State},
    };

    static ACTOR_1_ID: LazyLock<actor::Id> = LazyLock::new(|| actor::Id("actor_1".to_string()));
    static ACTOR_2_ID: LazyLock<actor::Id> = LazyLock::new(|| actor::Id("actor_2".to_string()));

    #[tokio::test]
    async fn intransitive_action_template_execution_works() {
        let actor: Arc<dyn ActorBase> = Arc::new(TestActor1::new(10));
        let actor_state_original: Arc<dyn ActorState> = Arc::new(TestActor1State { value: 10 });
        let actor_state_updated: Arc<dyn ActorState> = Arc::new(TestActor1State { value: 11 });
        let local_states = LocalStates(BTreeMap::from([(
            ACTOR_1_ID.clone(),
            LocalState {
                actor_state: actor_state_original.clone(),
            },
        )]));

        let mut actor_1_factory = MockActorFactory::new();
        let mut action_executor = MockActionExecutor::new();
        let mut actor_1_state_extractor = MockActorStateExtractor::new();

        let restored_actor = actor.clone();
        actor_1_factory
            .expect_restore_from_state()
            .withf(|actor_state| actor_1_state_having_value(actor_state, 10))
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
            .withf(|actor| actor_1_having_value(actor, 10))
            .once()
            .return_once(|_| actor_state_updated_clone);

        let executor = SimpleActionTemplateExecutor {
            action_executor,
            actor_factories: HashMap::from([(
                ACTOR_1_ID.clone(),
                Box::new(actor_1_factory) as Box<dyn ActorFactory>,
            )]),
            actor_state_extractors: HashMap::from([(
                ACTOR_1_ID.clone(),
                Box::new(actor_1_state_extractor) as Box<dyn ActorStateExtractor>,
            )]),
        };

        let execution_result = executor
            .execute(
                ActionTemplate {
                    performer_id: ACTOR_1_ID.clone(),
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

    #[tokio::test]
    async fn transitive_action_template_execution_works() {
        let actor_1: Arc<dyn ActorBase> = Arc::new(TestActor1::new(10));
        let actor_2: Arc<dyn ActorBase> = Arc::new(TestActor2::new(20));
        let actor_1_state_original: Arc<dyn ActorState> = Arc::new(TestActor1State { value: 10 });
        let actor_1_state_updated: Arc<dyn ActorState> = Arc::new(TestActor1State { value: 11 });
        let actor_2_state_original: Arc<dyn ActorState> = Arc::new(TestActor2State { value: 20 });
        let actor_2_state_updated: Arc<dyn ActorState> = Arc::new(TestActor2State { value: 21 });
        let local_states = LocalStates(BTreeMap::from([
            (
                ACTOR_1_ID.clone(),
                LocalState {
                    actor_state: actor_1_state_original.clone(),
                },
            ),
            (
                ACTOR_2_ID.clone(),
                LocalState {
                    actor_state: actor_2_state_original.clone(),
                },
            ),
        ]));

        let mut actor_1_factory = MockActorFactory::new();
        let mut actor_2_factory = MockActorFactory::new();
        let mut action_executor = MockActionExecutor::new();
        let mut actor_1_state_extractor = MockActorStateExtractor::new();
        let mut actor_2_state_extractor = MockActorStateExtractor::new();

        let restored_actor_1 = actor_1.clone();
        actor_1_factory
            .expect_restore_from_state()
            .withf(|actor_state| actor_1_state_having_value(actor_state, 10))
            .once()
            .return_once(|_| restored_actor_1);

        let restored_actor_2 = actor_2.clone();
        actor_2_factory
            .expect_restore_from_state()
            .withf(|actor_state| actor_2_state_having_value(actor_state, 20))
            .once()
            .return_once(|_| restored_actor_2);

        action_executor
            .expect_execute()
            .with(eq(Action::Transitive {
                performer: actor_1,
                receiver: actor_2,
                action: Box::new(|actor_1, actor_2| {
                    Box::pin(proxy_for_transitive_action(actor_1, actor_2))
                }),
            }))
            .once()
            .return_once(|_| ActionResult(None));

        let actor_1_state_updated_clone = actor_1_state_updated.clone();
        actor_1_state_extractor
            .expect_extract()
            .withf(|actor| actor_1_having_value(actor, 10))
            .once()
            .return_once(|_| actor_1_state_updated_clone);

        let actor_2_state_updated_clone = actor_2_state_updated.clone();
        actor_2_state_extractor
            .expect_extract()
            .withf(|actor| actor_2_having_value(actor, 20))
            .once()
            .return_once(|_| actor_2_state_updated_clone);

        let executor = SimpleActionTemplateExecutor {
            action_executor,
            actor_factories: HashMap::from([
                (
                    ACTOR_1_ID.clone(),
                    Box::new(actor_1_factory) as Box<dyn ActorFactory>,
                ),
                (
                    ACTOR_2_ID.clone(),
                    Box::new(actor_2_factory) as Box<dyn ActorFactory>,
                ),
            ]),
            actor_state_extractors: HashMap::from([
                (
                    ACTOR_1_ID.clone(),
                    Box::new(actor_1_state_extractor) as Box<dyn ActorStateExtractor>,
                ),
                (
                    ACTOR_2_ID.clone(),
                    Box::new(actor_2_state_extractor) as Box<dyn ActorStateExtractor>,
                ),
            ]),
        };

        let execution_result = executor
            .execute(
                ActionTemplate {
                    performer_id: ACTOR_1_ID.clone(),
                    label: "some_transitive_action",
                    action_type: ActionType::Transitive {
                        receiver_id: ACTOR_2_ID.clone(),
                        action: Box::new(|actor_1, actor_2| {
                            Box::pin(proxy_for_transitive_action(actor_1, actor_2))
                        }),
                    },
                },
                local_states,
            )
            .await;

        assert!(execution_result.action_result.0.is_none());
        assert_eq!(
            execution_result.local_states,
            LocalStates(BTreeMap::from([
                (
                    ACTOR_1_ID.clone(),
                    LocalState {
                        actor_state: actor_1_state_updated,
                    },
                ),
                (
                    ACTOR_2_ID.clone(),
                    LocalState {
                        actor_state: actor_2_state_updated,
                    },
                )
            ]))
        );
    }

    fn actor_1_state_having_value(actor_state: &Arc<dyn ActorState>, expected_value: u8) -> bool {
        ActorState::as_any(actor_state.as_ref())
            .downcast_ref::<TestActor1State>()
            .unwrap()
            .value
            == expected_value
    }

    fn actor_2_state_having_value(actor_state: &Arc<dyn ActorState>, expected_value: u8) -> bool {
        ActorState::as_any(actor_state.as_ref())
            .downcast_ref::<TestActor2State>()
            .unwrap()
            .value
            == expected_value
    }

    fn actor_1_having_value(actor: &Arc<dyn ActorBase>, expected_value: u8) -> bool {
        ActorBase::as_any(actor.as_ref())
            .downcast_ref::<TestActor1>()
            .unwrap()
            .get_value()
            == expected_value
    }

    fn actor_2_having_value(actor: &Arc<dyn ActorBase>, expected_value: u8) -> bool {
        ActorBase::as_any(actor.as_ref())
            .downcast_ref::<TestActor2>()
            .unwrap()
            .get_value()
            == expected_value
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
