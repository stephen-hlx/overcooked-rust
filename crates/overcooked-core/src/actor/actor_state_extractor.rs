use std::sync::Arc;

use crate::actor::{ActorBase, actor_state::ActorState};

#[mockall::automock]
#[async_trait::async_trait]
pub trait ActorStateExtractor: Sync {
    async fn extract(&self, actor: Arc<dyn ActorBase>) -> Arc<dyn ActorState>;
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        actor::{ActorBase, actor_state::ActorState, actor_state_extractor::ActorStateExtractor},
        test_utils::test_actors::{TestActor1, TestActor1State},
    };

    struct TestActor1StateExtractor;

    #[async_trait::async_trait]
    impl ActorStateExtractor for TestActor1StateExtractor {
        async fn extract(&self, actor: Arc<dyn ActorBase>) -> Arc<dyn ActorState> {
            Arc::new(TestActor1State {
                value: ActorBase::as_any(actor.as_ref())
                    .downcast_ref::<TestActor1>()
                    .unwrap()
                    .value
                    .get_value(),
            })
        }
    }

    #[tokio::test]
    async fn works() {
        let actor = TestActor1::new(10);
        let extractor = TestActor1StateExtractor;

        let binding = extractor.extract(Arc::new(actor)).await;
        assert_eq!(
            *ActorState::as_any(binding.as_ref())
                .downcast_ref::<TestActor1State>()
                .unwrap(),
            TestActor1State { value: 10 }
        );
    }
}
