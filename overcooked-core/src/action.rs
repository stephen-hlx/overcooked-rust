use std::{error::Error, pin::Pin, sync::Arc};

use crate::actor::ActorBase;

/// We may need to replace T1 with Box<T1> just to make sure
/// we can support more than 2 `ActorBase` implementations.
pub struct ActionDefinition<T1, T2>
where
    T1: ActorBase,
    T2: ActorBase,
{
    pub label: &'static str,
    pub action: Action<T1, T2>,
}

pub enum Action<T1, T2>
where
    T1: ActorBase,
    T2: ActorBase,
{
    Intransitive {
        performer: Box<T1>,
        action: Arc<
            dyn Fn(
                Box<T1>,
            )
                -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'static>>,
        >,
    },
    Transitive {
        performer: Box<T1>,
        receiver: Box<T2>,
        action: Arc<
            dyn Fn(
                Box<T1>,
                Box<T2>,
            )
                -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'static>>,
        >,
    },
}

#[cfg(test)]
mod tests {
    use std::{error::Error, sync::Arc};

    use crate::{
        action::Action,
        actor::ActorBase,
        test_utils::test_actor_states::{TestActor1, TestActor2},
    };

    pub struct DummyActionReceiver;
    impl ActorBase for DummyActionReceiver {}

    #[tokio::test]
    async fn intransitive_action_type_can_be_defined_and_triggered() {
        let actor = TestActor1;
        let intransitivie_action: Action<TestActor1, DummyActionReceiver> = Action::Intransitive {
            performer: Box::new(actor),
            action: Arc::new(|actor| Box::pin(proxy_for_intransitive_action(actor))),
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
        let transitivie_action: Action<TestActor1, TestActor2> = Action::Transitive {
            performer: Box::new(actor_1),
            receiver: Box::new(actor_2),
            action: Arc::new(|actor_1, actor_2| {
                Box::pin(proxy_for_transitive_action(actor_1, actor_2))
            }),
        };

        execute_action(transitivie_action).await.unwrap();
    }

    async fn proxy_for_transitive_action(
        test_actor_1: Box<TestActor1>,
        test_actor_2: Box<TestActor2>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(test_actor_1
            .do_something_to_test_actor_2(test_actor_2)
            .await
            .map_err(|e| Box::new(e))?)
    }

    async fn execute_action<T1, T2>(action: Action<T1, T2>) -> Result<(), Box<dyn Error>>
    where
        T1: ActorBase,
        T2: ActorBase,
    {
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
