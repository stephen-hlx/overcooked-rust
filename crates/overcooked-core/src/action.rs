use std::{collections::HashMap, error::Error, pin::Pin, sync::Arc};

use crate::{
    action::action_template_executor::{SimpleActionExecutor, SimpleActionTemplateExecutor},
    actor::{
        self, ActorBase, actor_factory::ActorFactory, actor_state_extractor::ActorStateExtractor,
    },
    global_state::GlobalState,
};

mod action_template_executor;

pub type IntransitiveAction = Arc<
    dyn Fn(
            Arc<dyn ActorBase>,
        )
            -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send>>> + Send + 'static>>
        + Send
        + Sync,
>;

pub type TransitiveAction = Arc<
    dyn Fn(
            Arc<dyn ActorBase>,
            Arc<dyn ActorBase>,
        )
            -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send>>> + Send + 'static>>
        + Send
        + Sync,
>;

#[derive(Clone)]
pub enum ActionType {
    Intransitive(IntransitiveAction),
    Transitive {
        receiver_id: actor::Id,
        action: TransitiveAction,
    },
}

#[derive(Clone)]
pub struct ActionTemplate {
    pub performer_id: actor::Id,
    pub label: &'static str,
    pub action_type: ActionType,
}

#[mockall::automock]
#[async_trait::async_trait]
pub trait ActionTemplateExecutor {
    async fn execute(&self, template: ActionTemplate, global_state: GlobalState)
    -> ExecutionResult;
}

#[derive(Debug)]
pub struct ActionResult(pub Option<Box<dyn Error + Send>>);

pub struct ExecutionResult {
    pub action_result: ActionResult,
    pub global_states: GlobalState,
}

pub fn create_executor(
    actor_factories: HashMap<actor::Id, Box<dyn ActorFactory>>,
    actor_state_extractors: HashMap<actor::Id, Box<dyn ActorStateExtractor>>,
) -> Box<dyn ActionTemplateExecutor> {
    Box::new(SimpleActionTemplateExecutor {
        action_executor: SimpleActionExecutor,
        actor_factories,
        actor_state_extractors,
    })
}

impl std::cmp::PartialEq for ActionTemplate {
    fn eq(&self, other: &Self) -> bool {
        self.performer_id == other.performer_id
            && self.label == other.label
            && match (&self.action_type, &other.action_type) {
                (ActionType::Intransitive(_), ActionType::Intransitive(_)) => true,
                (
                    ActionType::Transitive {
                        receiver_id: self_receiver_id,
                        action: _,
                    },
                    ActionType::Transitive {
                        receiver_id: other_receiver_id,
                        action: _,
                    },
                ) => self_receiver_id == other_receiver_id,
                _ => false,
            }
    }
}

impl std::cmp::Eq for ActionTemplate {}

impl std::hash::Hash for ActionTemplate {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.performer_id.hash(state);
        self.label.hash(state);
        match &self.action_type {
            ActionType::Intransitive(_) => {}
            ActionType::Transitive {
                receiver_id,
                action: _,
            } => receiver_id.hash(state),
        }
    }
}

impl std::fmt::Debug for ActionTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionTemplate")
            .field("performer_id", &self.performer_id)
            .field("label", &self.label)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        error::Error,
        sync::{Arc, LazyLock},
    };

    use crate::{
        action::ActionType,
        actor::{self, ActorBase},
    };

    use super::ActionTemplate;

    #[test]
    fn action_template_can_be_added_to_a_set() {
        let _ = HashSet::from([(ActionTemplate {
            performer_id: actor::Id("actor".to_string()),
            label: "some_intransitive_action",
            action_type: ActionType::Intransitive(Arc::new(|actor| {
                Box::pin(proxy_for_intransitive_action(actor))
            })),
        })]);
    }

    static ACTOR_1_ID: LazyLock<actor::Id> = LazyLock::new(|| actor::Id("actor_1".to_string()));
    static ACTOR_2_ID: LazyLock<actor::Id> = LazyLock::new(|| actor::Id("actor_2".to_string()));

    #[test]
    fn action_template_eq_works() {
        // Intransitive
        assert_eq!(
            ActionTemplate {
                performer_id: ACTOR_1_ID.clone(),
                label: "some_intransitive_action",
                action_type: intransitive_action_type(),
            },
            ActionTemplate {
                performer_id: ACTOR_1_ID.clone(),
                label: "some_intransitive_action",
                action_type: intransitive_action_type(),
            }
        );

        assert_ne!(
            ActionTemplate {
                performer_id: ACTOR_1_ID.clone(),
                label: "some_intransitive_action",
                action_type: intransitive_action_type(),
            },
            ActionTemplate {
                performer_id: ACTOR_1_ID.clone(),
                label: "some_other_intransitive_action",
                action_type: intransitive_action_type(),
            }
        );

        assert_ne!(
            ActionTemplate {
                performer_id: ACTOR_1_ID.clone(),
                label: "some_intransitive_action",
                action_type: intransitive_action_type(),
            },
            ActionTemplate {
                performer_id: ACTOR_2_ID.clone(),
                label: "some_intransitive_action",
                action_type: intransitive_action_type(),
            }
        );

        // Transitive
        assert_eq!(
            ActionTemplate {
                performer_id: ACTOR_1_ID.clone(),
                label: "some_transitive_action",
                action_type: transitive_action_type(ACTOR_2_ID.clone()),
            },
            ActionTemplate {
                performer_id: ACTOR_1_ID.clone(),
                label: "some_transitive_action",
                action_type: transitive_action_type(ACTOR_2_ID.clone()),
            }
        );

        assert_ne!(
            ActionTemplate {
                performer_id: ACTOR_1_ID.clone(),
                label: "some_transitive_action",
                action_type: transitive_action_type(ACTOR_2_ID.clone()),
            },
            ActionTemplate {
                performer_id: ACTOR_1_ID.clone(),
                label: "some_other_transitive_action",
                action_type: transitive_action_type(ACTOR_2_ID.clone()),
            }
        );

        assert_ne!(
            ActionTemplate {
                performer_id: ACTOR_1_ID.clone(),
                label: "some_transitive_action",
                action_type: transitive_action_type(ACTOR_2_ID.clone()),
            },
            ActionTemplate {
                performer_id: ACTOR_2_ID.clone(),
                label: "some_transitive_action",
                action_type: transitive_action_type(ACTOR_1_ID.clone()),
            }
        );
    }

    fn intransitive_action_type() -> ActionType {
        ActionType::Intransitive(Arc::new(|actor| {
            Box::pin(proxy_for_intransitive_action(actor))
        }))
    }

    fn transitive_action_type(receiver_id: actor::Id) -> ActionType {
        ActionType::Transitive {
            receiver_id,
            action: Arc::new(|performer, receiver| {
                Box::pin(proxy_for_transitive_action(performer, receiver))
            }),
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
