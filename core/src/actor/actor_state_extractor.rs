use crate::actor::{ActorBase, actor_state::ActorState};

pub trait ActorStateExtractor {
    fn extract<T: ActorBase>(&self, actor: Box<T>) -> Box<dyn ActorState>
    where
        Self: Sized;
}
