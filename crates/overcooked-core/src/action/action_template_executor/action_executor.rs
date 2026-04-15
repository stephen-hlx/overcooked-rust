use super::{Action, ActionExecutor, ActionResult};

pub(super) struct SimpleActionExecutor;

#[async_trait::async_trait]
impl ActionExecutor for SimpleActionExecutor {
    async fn execute(&self, action: Action) -> ActionResult {
        if let Err(err) = match action {
            Action::Intransitive { performer, action } => action(performer),
            Action::Transitive {
                performer,
                receiver,
                action,
            } => action(performer, receiver),
        }
        .await
        {
            ActionResult(Some(err))
        } else {
            ActionResult(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, sync::Arc};

    use crate::{
        actor::ActorBase,
        test_utils::test_actors::{TestActor1, TestActor2},
    };

    use super::*;

    #[tokio::test]
    async fn can_execute_intrasnsitive_action() {
        let executor = SimpleActionExecutor;
        let test_actor_1 = Arc::new(TestActor1::new(0));
        let action = Action::Intransitive {
            performer: test_actor_1.clone(),
            action: Arc::new(|actor| Box::pin(proxy_for_intransitive_action(actor))),
        };

        assert!(executor.execute(action).await.0.is_none());

        assert_eq!(test_actor_1.get_value(), 1);
    }

    async fn proxy_for_intransitive_action(
        test_actor_1: Arc<dyn ActorBase>,
    ) -> Result<(), Box<dyn Error + Send>> {
        Ok(ActorBase::as_any(test_actor_1.as_ref())
            .downcast_ref::<TestActor1>()
            .unwrap()
            .increase_inner_value_by_one()
            .await
            .map_err(|e| {
                let err: Box<dyn Error + Send> = Box::new(e);
                err
            })?)
    }

    #[tokio::test]
    async fn can_execute_transitive_action() {
        let executor = SimpleActionExecutor;
        let test_actor_1 = Arc::new(TestActor1::new(0));
        let test_actor_2 = Arc::new(TestActor2::new(5));
        let action = Action::Transitive {
            performer: test_actor_1.clone(),
            receiver: test_actor_2.clone(),
            action: Arc::new(|action_performer, action_receiver| {
                Box::pin(proxy_for_transitive_action(
                    action_performer,
                    action_receiver,
                ))
            }),
        };

        assert!(executor.execute(action).await.0.is_none());

        assert_eq!(test_actor_1.get_value(), 0);
        assert_eq!(test_actor_2.get_value(), 4);
    }

    async fn proxy_for_transitive_action(
        test_actor_1: Arc<dyn ActorBase>,
        test_actor_2: Arc<dyn ActorBase>,
    ) -> Result<(), Box<dyn Error + Send>> {
        Ok(ActorBase::as_any(test_actor_1.as_ref())
            .downcast_ref::<TestActor1>()
            .unwrap()
            .decrease_test_actor_2_value_by_one(
                ActorBase::as_any(test_actor_2.as_ref())
                    .downcast_ref::<TestActor2>()
                    .unwrap(),
            )
            .await
            .map_err(|e| {
                let err: Box<dyn Error + Send> = Box::new(e);
                err
            })?)
    }
}
