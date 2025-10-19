use dyn_clone::DynClone;

use crate::{dyn_ord::DynOrd, dyn_partial_eq::DynPartialEq, dyn_partial_ord::DynPartialOrd};

/// The state of an actor in a system that consists of multiple actors.
pub trait ActorState: DynPartialEq + DynPartialOrd + DynOrd + DynClone + std::fmt::Debug {}

pub trait Temp: Ord {}
