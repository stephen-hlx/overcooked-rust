mod actor_factory;
pub mod actor_state;
mod actor_state_extractor;
pub mod actor_state_transformer_config;
pub mod local_state;

/// Id of an actor
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(pub String);

pub trait ActorBase {}
