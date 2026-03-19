use std::{any::Any, error::Error, pin::Pin, sync::Arc};

use crate::actor;

mod action_executor;

pub type IntransitiveAction = Box<
    dyn Fn(
        Arc<dyn Any + Send + Sync>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'static>>,
>;

pub type TransitiveAction = Box<
    dyn Fn(
        Arc<dyn Any + Send + Sync>,
        Arc<dyn Any + Send + Sync>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'static>>,
>;

/// We may need to replace T1 with Box<T1> just to make sure
/// we can support more than 2 `ActorBase` implementations.
pub struct ActionDefinition {
    pub label: &'static str,
    pub action: Action,
}

pub enum Action {
    Intransitive {
        performer: Arc<dyn Any + Send + Sync>,
        action: IntransitiveAction,
    },
    Transitive {
        performer: Arc<dyn Any + Send + Sync>,
        receiver: Arc<dyn Any + Send + Sync>,
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
