use std::sync::Arc;

use super::{Action, ActionExecutor, ActionResult};

pub struct SimpleActionExecutor;

#[async_trait::async_trait]
impl ActionExecutor for SimpleActionExecutor {
    async fn execute(&self, action: Action) -> ActionResult {
        if let Err(err) = match action {
            Action::Intransitive { performer, action } => action.0(performer),
            Action::Transitive {
                performer,
                receiver,
                action,
            } => action.0(performer, receiver),
        }
        .await
        {
            ActionResult(Some(Arc::from(err)))
        } else {
            ActionResult(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        action::{IntransitiveAction, TransitiveAction},
        test_utils::test_actors::{
            TestActor1, TestActor2, test_actor1_decrease_test_actor_2_value_by_one_test_actor2,
            test_actor1_increase_inner_value_by_one,
        },
    };

    use super::*;

    #[tokio::test]
    async fn can_execute_intrasnsitive_action() {
        let executor = SimpleActionExecutor;
        let test_actor_1 = Arc::new(TestActor1::new(0));
        let action = Action::Intransitive {
            performer: test_actor_1.clone(),
            action: IntransitiveAction::of(test_actor1_increase_inner_value_by_one),
        };

        assert!(executor.execute(action).await.0.is_none());

        assert_eq!(test_actor_1.get_value(), 1);
    }

    #[tokio::test]
    async fn can_execute_transitive_action() {
        let executor = SimpleActionExecutor;
        let test_actor_1 = Arc::new(TestActor1::new(0));
        let test_actor_2 = Arc::new(TestActor2::new(5));
        let action = Action::Transitive {
            performer: test_actor_1.clone(),
            receiver: test_actor_2.clone(),
            action: TransitiveAction::of(
                test_actor1_decrease_test_actor_2_value_by_one_test_actor2,
            ),
        };

        assert!(executor.execute(action).await.0.is_none());

        assert_eq!(test_actor_1.get_value(), 0);
        assert_eq!(test_actor_2.get_value(), 4);
    }
}
