use std::{error::Error, pin::Pin, sync::Arc};

use crate::{
    actor::{self, ActorBase},
    global_state::{GlobalState, LocalStates},
};

mod action_template_executor;

pub type IntransitiveAction = Box<
    dyn Fn(
            Arc<dyn ActorBase>,
        ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'static>>
        + Send,
>;

pub type TransitiveAction = Box<
    dyn Fn(
            Arc<dyn ActorBase>,
            Arc<dyn ActorBase>,
        ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'static>>
        + Send,
>;

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
trait ActionTemplateExecutor {
    async fn execute(&self, template: ActionTemplate, local_states: LocalStates)
    -> ExecutionResult;
}

pub struct ActionResult(pub Option<Box<dyn Error>>);

pub struct ExecutionResult {
    pub action_result: ActionResult,
    pub local_states: LocalStates,
}
