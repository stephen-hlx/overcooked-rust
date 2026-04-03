use std::sync::Arc;

use crate::actor::{ActorBase, actor_state::ActorState};

pub trait ActorFactory {
    fn restore_from_state(&self, actor_state: Arc<dyn ActorState>) -> Arc<dyn ActorBase>;
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        actor::{ActorBase, actor_factory::ActorFactory, actor_state::ActorState},
        test_utils::test_actors::{TestActor1, TestActor1State},
    };

    struct TestActor1Factory;

    impl ActorFactory for TestActor1Factory {
        fn restore_from_state(&self, actor_state: Arc<dyn ActorState>) -> Arc<dyn ActorBase> {
            Arc::new(TestActor1::new(
                ActorState::as_any(actor_state.as_ref())
                    .downcast_ref::<TestActor1State>()
                    .unwrap()
                    .value,
            ))
        }
    }

    #[test]
    fn works() {
        let state = TestActor1State { value: 10 };
        let factory = TestActor1Factory;

        assert_eq!(
            ActorBase::as_any(factory.restore_from_state(Arc::new(state)).as_ref())
                .downcast_ref::<TestActor1>()
                .unwrap()
                .get_value(),
            TestActor1::new(10).get_value()
        );
    }
}
