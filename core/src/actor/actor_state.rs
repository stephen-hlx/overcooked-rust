use dyn_clone::DynClone;

use crate::dyn_partial_eq::DynPartialEq;

/// The state of an actor in a system that consists of multiple actors.
pub trait ActorState: DynPartialEq + DynClone + std::fmt::Debug {}
