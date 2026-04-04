use std::{error::Error, pin::Pin, sync::Arc};

use crate::{
    actor::{self, ActorBase},
    global_state::LocalStates,
};

mod action_template_executor;

pub type IntransitiveAction = Box<
    dyn Fn(
            Arc<dyn ActorBase>,
        )
            -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send>>> + Send + 'static>>
        + Send,
>;

pub type TransitiveAction = Box<
    dyn Fn(
            Arc<dyn ActorBase>,
            Arc<dyn ActorBase>,
        )
            -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send>>> + Send + 'static>>
        + Send,
>;

pub enum ActionType {
    Intransitive(IntransitiveAction),
    Transitive {
        receiver_id: actor::Id,
        action: TransitiveAction,
    },
}

pub struct ActionTemplate {
    pub performer_id: actor::Id,
    pub label: &'static str,
    pub action_type: ActionType,
}

#[async_trait::async_trait]
trait ActionTemplateExecutor {
    async fn execute(&self, template: ActionTemplate, local_states: LocalStates)
    -> ExecutionResult;
}

pub struct ActionResult(pub Option<Box<dyn Error + Send>>);

pub struct ExecutionResult {
    pub action_result: ActionResult,
    pub local_states: LocalStates,
}
