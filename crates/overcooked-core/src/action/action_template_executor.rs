use std::sync::Arc;

use crate::{
    action::{ActionResult, IntransitiveAction, TransitiveAction},
    actor::ActorBase,
};

mod action;
mod action_executor;

#[async_trait::async_trait]
trait ActionExecutor {
    async fn execute(&self, action: Action) -> ActionResult;
}

enum Action {
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
