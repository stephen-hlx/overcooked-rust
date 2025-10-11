mod actor_state;
pub mod local_state;

/// Id of an actor
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(String);
