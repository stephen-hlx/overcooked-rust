/// TODO: is there a way to make the use of the generated fn more ergonomic?

#[macro_export]
macro_rules! intransitive_action {
    ($actor_type:ident, $method:ident) => {
        paste::paste! {
            pub async fn [<$actor_type:snake _ $method>](
                actor: ::std::sync::Arc<dyn crate::actor::ActorBase>,
            ) -> Result<(), Box<dyn ::std::error::Error + Send>> {
                Ok(crate::actor::ActorBase::as_any(actor.as_ref())
                    .downcast_ref::<$actor_type>()
                    .unwrap()
                    .$method()
                    .await
                    .map_err(|e| {
                        let err: Box<dyn ::std::error::Error + Send> = Box::new(e);
                        err
                    })?)
            }
        }
    };
}

#[macro_export]
macro_rules! transitive_action {
    ($action_performer_type:ident, $method:ident, $action_receiver_type:ident) => {
        paste::paste! {
            pub async fn [<$action_performer_type:snake _ $method _ $action_receiver_type:snake>](
                action_performer: ::std::sync::Arc<dyn crate::actor::ActorBase>,
                action_receiver: ::std::sync::Arc<dyn crate::actor::ActorBase>,
            ) -> Result<(), Box<dyn ::std::error::Error + Send>> {
                Ok(crate::actor::ActorBase::as_any(action_performer.as_ref())
                    .downcast_ref::<$action_performer_type>()
                    .unwrap()
                    .$method(
                        crate::actor::ActorBase::as_any(action_receiver.as_ref())
                        .downcast_ref::<$action_receiver_type>()
                        .unwrap())
                    .await
                    .map_err(|e| {
                        let err: Box<dyn ::std::error::Error + Send> = Box::new(e);
                        err
                    })?)
            }
        }
    };
}
