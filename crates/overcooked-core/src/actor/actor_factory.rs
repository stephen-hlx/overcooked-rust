use crate::actor::{ActorBase, actor_state::ActorState};

pub trait ActorFactory {
    fn restore_from_state(actor_state: Box<dyn ActorState>) -> Box<dyn ActorBase>
    where
        Self: Sized;
}
