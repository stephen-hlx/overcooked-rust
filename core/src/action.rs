use std::{error::Error, pin::Pin, sync::Arc};

use crate::actor::ActorBase;

pub struct Action<T1, T2>
where
    T1: ActorBase,
    T2: ActorBase,
{
    pub label: &'static str,
    pub action_type: ActionType<T1, T2>,
}

pub enum ActionType<T1, T2>
where
    T1: ActorBase,
    T2: ActorBase,
{
    Intransitive {
        performer: Box<T1>,
        action: Arc<
            dyn Fn(
                Box<T1>,
            )
                -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'static>>,
        >,
    },
    Transitive {
        performer: Box<T1>,
        receiver: Box<T2>,
        action: Arc<
            dyn Fn(
                Box<T1>,
                Box<T2>,
            )
                -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'static>>,
        >,
    },
}

pub struct DummyActionReceiver;
impl ActorBase for DummyActionReceiver {}

#[cfg(test)]
mod tests {
    use std::{error::Error, pin::Pin, sync::Arc};

    use crate::{
        action::{Action, ActionType, DummyActionReceiver},
        actor::ActorBase,
        test_utils::test_actor_states::{TestActor1, TestActor2},
    };

    #[tokio::test]
    async fn action_type_can_be_defined_and_triggered() {
        let actor = TestActor1;
        let intransitivie_action: ActionType<TestActor1, DummyActionReceiver> =
            ActionType::Intransitive {
                performer: Box::new(actor),
                action: Arc::new(|actor| Box::pin(proxy(actor))),
            };
    }

    async fn proxy(test_actor_1: Box<TestActor1>) -> Result<(), Box<dyn Error>> {
        Ok(test_actor_1
            .do_something_on_its_own()
            .await
            .map_err(|e| Box::new(e))?)
    }

    struct TestStruct {
        action: Arc<
            dyn Fn(
                Box<String>,
                Box<String>,
            )
                -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'static>>,
            // dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'static>>,
            // dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
        >,
    }

    #[derive(Debug, thiserror::Error)]
    #[error("asdf")]
    struct MyError;

    #[tokio::test]
    async fn action_can_be_constructed() {
        let my_actor_1 = Box::new(String::from("a"));
        let my_actor_2 = Box::new(String::from("b"));

        // let f: &dyn Fn(Box<dyn ActorBase>, Box<dyn ActorBase>) -> Result<(), Box<dyn Error>> =
        //     async |my_actor_1: Box<MyActor1>, my_actor_2: Box<MyActor2>| {
        //         my_actor_1.do_something(my_actor_2).await
        //     };

        let s = TestStruct {
            action: Arc::new(|s1, s2| Box::pin(f(s1, s2))),
        };

        (s.action)(my_actor_1, my_actor_2).await;
    }

    // fn new_struct<F, Fut>(f: F) -> TestStruct
    // where
    //     F: Fn() -> Fut + Send + 'static,
    //     Fut: Future<Output = ()> + Send + 'static,
    // {
    //     TestStruct {
    //         action: Arc::new(move || Box::pin(f())),
    //     }
    // }

    async fn f(s1: Box<String>, s2: Box<String>) -> Result<(), Box<dyn Error>> {
        println!("{s1}");
        println!("{s2}");
        Ok(())
    }
    async fn f1() -> Result<(), Box<dyn Error>> {
        // Err(Box::new(MyError))
        Err(MyError)?
    }
    async fn f2() {}
}
