use crate::actor::actor_state::ActorState;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct MyActorState {
    pub value: u8,
}

impl ActorState for MyActorState {}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct MyActorState2 {
    pub value: u8,
}

impl ActorState for MyActorState2 {}
