mod actor_factory;
mod actor_state;
mod actor_state_extractor;
pub mod local_state;

/// Id of an actor
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(String);

pub trait ActorBase {}
