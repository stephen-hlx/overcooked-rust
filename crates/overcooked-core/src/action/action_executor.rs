use std::error::Error;

use crate::action::{Action, ActionExecutor, ActionResult};

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
        action::{Action, ActionExecutor, action_executor::SimpleActionExecutor},
        actor::ActorBase,
        test_utils::test_actors::{TestActor1, TestActor2},
    };

    #[tokio::test]
    async fn can_execute_intrasnsitive_action() {
        let executor = SimpleActionExecutor;
        let test_actor_1 = Arc::new(TestActor1::new(0));
        let intransitivie_action: Action = Action::Intransitive {
            performer: test_actor_1.clone(),
            action: Box::new(|actor| Box::pin(proxy_for_intransitive_action(actor))),
        };

        assert!(executor.execute(intransitivie_action).await.0.is_none());

        assert_eq!(test_actor_1.get_value(), 1);
    }

    async fn proxy_for_intransitive_action(
        test_actor_1: Arc<dyn ActorBase + Send + Sync>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(test_actor_1
            .as_any()
            .downcast_ref::<TestActor1>()
            .unwrap()
            .increase_inner_value_by_one()
            .await
            .map_err(|e| Box::new(e))?)
    }

    #[tokio::test]
    async fn can_execute_transitive_action() {
        let executor = SimpleActionExecutor;
        let test_actor_1 = Arc::new(TestActor1::new(0));
        let test_actor_2 = Arc::new(TestActor2::new(5));
        let transitivie_action: Action = Action::Transitive {
            performer: test_actor_1.clone(),
            receiver: test_actor_2.clone(),
            action: Box::new(|action_performer, action_receiver| {
                Box::pin(proxy_for_transitive_action(
                    action_performer,
                    action_receiver,
                ))
            }),
        };

        assert!(executor.execute(transitivie_action).await.0.is_none());

        assert_eq!(test_actor_1.get_value(), 0);
        assert_eq!(test_actor_2.get_value(), 4);
    }

    async fn proxy_for_transitive_action(
        test_actor_1: Arc<dyn ActorBase + Send + Sync>,
        test_actor_2: Arc<dyn ActorBase + Send + Sync>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(test_actor_1
            .as_any()
            .downcast_ref::<TestActor1>()
            .unwrap()
            .decrease_test_actor_2_value_by_one(
                test_actor_2.as_any().downcast_ref::<TestActor2>().unwrap(),
            )
            .await
            .map_err(|e| Box::new(e))?)
    }
}
