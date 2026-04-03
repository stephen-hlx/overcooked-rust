use std::sync::Arc;

use crate::actor::{ActorBase, actor_state::ActorState};

pub trait ActorStateExtractor: Send + Sync {
    fn extract(&self, actor: Arc<dyn ActorBase>) -> Arc<dyn ActorState>;
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        actor::{ActorBase, actor_state::ActorState, actor_state_extractor::ActorStateExtractor},
        test_utils::test_actors::{TestActor1, TestActor1State},
    };

    struct TestActor1StateExtractor;

    impl ActorStateExtractor for TestActor1StateExtractor {
        fn extract(&self, actor: Arc<dyn ActorBase>) -> Arc<dyn ActorState> {
            Arc::new(TestActor1State {
                value: ActorBase::as_any(actor.as_ref())
                    .downcast_ref::<TestActor1>()
                    .unwrap()
                    .value
                    .get_value(),
            })
        }
    }

    #[test]
    fn works() {
        let actor = TestActor1::new(10);
        let extractor = TestActor1StateExtractor;

        let binding = extractor.extract(Arc::new(actor));
        assert_eq!(
            *ActorState::as_any(binding.as_ref())
                .downcast_ref::<TestActor1State>()
                .unwrap(),
            TestActor1State { value: 10 }
        );
    }
}
