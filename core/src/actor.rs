pub mod actor_state;
pub mod local_state;

/// Id of an actor
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(pub String);

pub trait ActorBase {}
