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

#[derive(Debug, PartialEq)]
pub struct LabelledAction {
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

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Intransitive {
                    performer: l_performer,
                    action: _,
                },
                Self::Intransitive {
                    performer: r_performer,
                    action: _,
                },
            ) => l_performer.dyn_eq(r_performer.as_ref()),
            (
                Self::Transitive {
                    performer: l_performer,
                    receiver: l_receiver,
                    action: _,
                },
                Self::Transitive {
                    performer: r_performer,
                    receiver: r_receiver,
                    action: _,
                },
            ) => l_performer.dyn_eq(r_performer.as_ref()) && l_receiver.dyn_eq(r_receiver.as_ref()),
            _ => false,
        }
    }
}

impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Intransitive {
                performer,
                action: _,
            } => f
                .debug_struct("Intransitive")
                .field("performer", performer)
                .finish(),
            Self::Transitive {
                performer,
                receiver,
                action: _,
            } => f
                .debug_struct("Transitive")
                .field("performer", performer)
                .field("receiver", receiver)
                .finish(),
        }
    }
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

#[cfg(test)]
mod tests {
    use std::{error::Error, sync::Arc};

    use test_case::test_case;

    use crate::{
        action::Action,
        actor::ActorBase,
        test_utils::test_actors::{TestActor1, TestActor2},
    };

    #[test_case(true, TestActor1::new(1), TestActor1::new(1))]
    #[test_case(false, TestActor1::new(1), TestActor1::new(2))]
    #[test_case(false, TestActor1::new(1), TestActor2::new(1))]
    fn intransitive_action_is_comparable_by_actor(
        expected: bool,
        actor_1: impl ActorBase + Send + Sync,
        actor_2: impl ActorBase + Send + Sync,
    ) {
        assert_eq!(
            Action::Intransitive {
                performer: Arc::new(actor_1),
                action: Box::new(|actor| Box::pin(proxy_for_intransitive_action(actor)))
            } == Action::Intransitive {
                performer: Arc::new(actor_2),
                action: Box::new(|actor| Box::pin(proxy_for_intransitive_action(actor)))
            },
            expected
        );
    }

    #[rustfmt::skip]
    #[test_case(true, TestActor1::new(1), TestActor2::new(1), TestActor1::new(1), TestActor2::new(1))]
    #[test_case(false, TestActor1::new(1), TestActor2::new(1), TestActor2::new(1), TestActor1::new(1))]
    #[test_case(false, TestActor1::new(1), TestActor2::new(1), TestActor1::new(2), TestActor2::new(2))]
    fn transitive_action_is_comparable_by_actor(
        expected: bool,
        performer_1: impl ActorBase + Send + Sync,
        receiver_1: impl ActorBase + Send + Sync,
        performer_2: impl ActorBase + Send + Sync,
        receiver_2: impl ActorBase + Send + Sync,
    ) {
        assert_eq!(
            Action::Transitive {
                performer: Arc::new(performer_1),
                receiver: Arc::new(receiver_1),
                action: Box::new(|performer, receiver| Box::pin(proxy_for_transitive_action(
                    performer, receiver,
                )))
            } == Action::Transitive {
                performer: Arc::new(performer_2),
                receiver: Arc::new(receiver_2),
                action: Box::new(|performer, receiver| Box::pin(proxy_for_transitive_action(
                    performer, receiver,
                )))
            },
            expected
        );
    }

    async fn proxy_for_intransitive_action(
        _: Arc<dyn ActorBase + Send + Sync>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn proxy_for_transitive_action(
        _: Arc<dyn ActorBase + Send + Sync>,
        _: Arc<dyn ActorBase + Send + Sync>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
