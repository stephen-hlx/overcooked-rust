use dyn_clone::DynClone;

use crate::derives::{
    dyn_hash::DynHash, dyn_ord::DynOrd, dyn_partial_eq::DynPartialEq,
    dyn_partial_ord::DynPartialOrd,
};

/// The state of an actor in a system that consists of multiple actors.
pub trait ActorState:
    DynPartialEq + DynPartialOrd + DynOrd + DynHash + DynClone + std::fmt::Debug
{
}

// dyn_hash::hash_trait_object!(ActorState);
