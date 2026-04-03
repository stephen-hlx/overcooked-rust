use std::{any::Any, sync::Arc};

use dyn_clone::DynClone;

use crate::derives::{
    dyn_hash::DynHash, dyn_ord::DynOrd, dyn_partial_eq::DynPartialEq,
    dyn_partial_ord::DynPartialOrd,
};

pub trait ActorState:
    Any + DynPartialEq + DynPartialOrd + DynOrd + DynHash + DynClone + std::fmt::Debug + Send
{
    fn as_any(&self) -> &dyn Any;
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any>;
}

#[macro_export]
macro_rules! impl_actor_state {
    ($t:ty) => {
        impl ActorState for $t {
            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }
            fn as_any_arc(self: Arc<Self>) -> Arc<dyn ::std::any::Any> {
                self
            }
        }
    };
}
