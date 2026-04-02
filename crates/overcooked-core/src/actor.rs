use std::{any::Any, sync::Arc};

mod actor_factory;
pub mod actor_state;
mod actor_state_extractor;
pub mod actor_state_transformer_config;
pub mod local_state;

/// Id of an actor
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(pub String);

pub trait ActorBase: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any>;
}

#[macro_export]
macro_rules! impl_actor_base {
    ($t:ty) => {
        impl ActorBase for $t {
            // self here is ALWAYS &$t — no ambiguity possible
            fn as_any(&self) -> &dyn ::std::any::Any {
                self
            }
            fn as_any_arc(self: Arc<Self>) -> Arc<dyn ::std::any::Any> {
                self
            }
        }
    };
}
