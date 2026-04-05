use super::Action;

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

#[cfg(test)]
mod tests {
    use std::{error::Error, sync::Arc};

    use test_case::test_case;

    use crate::{
        actor::ActorBase,
        test_utils::test_actors::{TestActor1, TestActor2},
    };

    use super::*;

    #[test_case(true, TestActor1::new(1), TestActor1::new(1))]
    #[test_case(false, TestActor1::new(1), TestActor1::new(2))]
    #[test_case(false, TestActor1::new(1), TestActor2::new(1))]
    fn intransitive_action_is_comparable_by_actor(
        expected: bool,
        actor_1: impl ActorBase,
        actor_2: impl ActorBase,
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
        performer_1: impl ActorBase,
        receiver_1: impl ActorBase,
        performer_2: impl ActorBase,
        receiver_2: impl ActorBase,
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
