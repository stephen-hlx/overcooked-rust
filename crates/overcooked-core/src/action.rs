use std::{error::Error, pin::Pin, sync::Arc};

use crate::actor::{self, ActorBase};

mod action_executor;

pub type IntransitiveAction = Box<
    dyn Fn(
            Arc<dyn ActorBase + Send + Sync>,
        ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'static>>
        + Send,
>;

pub type TransitiveAction = Box<
    dyn Fn(
            Arc<dyn ActorBase + Send + Sync>,
            Arc<dyn ActorBase + Send + Sync>,
        ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'static>>
        + Send,
>;

/// We may need to replace T1 with Box<T1> just to make sure
/// we can support more than 2 `ActorBase` implementations.
pub struct ActionDefinition {
    pub label: &'static str,
    pub action: Action,
}

pub enum Action {
    Intransitive {
        performer: Arc<dyn ActorBase + Send + Sync>,
        action: IntransitiveAction,
    },
    Transitive {
        performer: Arc<dyn ActorBase + Send + Sync>,
        receiver: Arc<dyn ActorBase + Send + Sync>,
        action: TransitiveAction,
    },
}

pub enum ActionType {
    Intransitive(IntransitiveAction),
    Transitive {
        receiver_id: actor::Id,
        transitive_action: TransitiveAction,
    },
}

pub struct ActionTemplate {
    pub actor_performer_id: actor::Id,
    pub label: &'static str,
    pub action_type: ActionType,
}

#[async_trait::async_trait]
trait ActionExecutor {
    async fn execute(&self, action: Action) -> ActionResult;
}
pub(super) struct ActionResult(Option<Box<dyn Error>>);
