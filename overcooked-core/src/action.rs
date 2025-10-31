use std::{any::Any, error::Error, pin::Pin, sync::Arc};

/// We may need to replace T1 with Box<T1> just to make sure
/// we can support more than 2 `ActorBase` implementations.
pub struct ActionDefinition {
    pub label: &'static str,
    pub action: Action,
}

pub enum Action {
    Intransitive {
        performer: Box<dyn Any + Send>,
        action: Arc<
            dyn Fn(
                Box<dyn Any>,
            )
                -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'static>>,
        >,
    },
    Transitive {
        performer: Box<dyn Any + Send>,
        receiver: Box<dyn Any + Send>,
        action: Arc<
            dyn Fn(
                Box<dyn Any + Send>,
                Box<dyn Any + Send>,
            )
                -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'static>>,
        >,
    },
}

/// The two proxy methods defined here are implemented with two different
/// interfaces. They both are the candidate of the potential proc macro that
/// will be used to simplify the use of the `Action`.
#[cfg(test)]
mod tests {
    use std::{any::Any, error::Error, sync::Arc};

    use crate::{
        action::Action,
        test_utils::test_actor_states::{TestActor1, TestActor2},
    };

    #[tokio::test]
    async fn intransitive_action_type_can_be_defined_and_triggered() {
        let actor = TestActor1;
        let intransitivie_action: Action = Action::Intransitive {
            performer: Box::new(actor),
            action: Arc::new(|actor| {
                let typed_actor = actor.downcast_ref::<TestActor1>().unwrap().clone();
                Box::pin(proxy_for_intransitive_action(Box::new(typed_actor)))
            }),
        };

        execute_action(intransitivie_action).await.unwrap();
    }

    async fn proxy_for_intransitive_action(
        test_actor_1: Box<TestActor1>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(test_actor_1
            .do_something_on_its_own()
            .await
            .map_err(|e| Box::new(e))?)
    }

    #[tokio::test]
    async fn transitive_action_type_can_be_defined_and_triggered() {
        let actor_1 = TestActor1;
        let actor_2 = TestActor2;
        let transitivie_action: Action = Action::Transitive {
            performer: Box::new(actor_1),
            receiver: Box::new(actor_2),
            action: Arc::new(|actor_1, actor_2| {
                Box::pin(proxy_for_transitive_action(actor_1, actor_2))
            }),
        };

        execute_action(transitivie_action).await.unwrap();
    }

    async fn proxy_for_transitive_action(
        test_actor_1: Box<dyn Any + Send>,
        test_actor_2: Box<dyn Any + Send>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(test_actor_1
            .downcast_ref::<TestActor1>()
            .unwrap()
            .do_something_to_test_actor_2(
                test_actor_2
                    .downcast_ref::<TestActor2>()
                    .unwrap()
                    .to_owned(),
            )
            .await
            .map_err(|e| Box::new(e))?)
    }

    async fn execute_action(action: Action) -> Result<(), Box<dyn Error>> {
        match action {
            Action::Intransitive { performer, action } => action(performer).await,
            Action::Transitive {
                performer,
                receiver,
                action,
            } => action(performer, receiver).await,
        }
    }
}
