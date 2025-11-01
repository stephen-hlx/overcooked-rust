use crate::actor::{ActorBase, actor_state::ActorState};

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

pub struct TestActor1;
pub struct TestActor2;
pub struct TestActor3;

impl ActorBase for TestActor1 {}
impl ActorBase for TestActor2 {}
impl ActorBase for TestActor3 {}

impl TestActor1 {
    pub async fn do_something(&self, _: Box<TestActor2>) -> Result<(), TestActor1Error> {
        println!("test_actor_1.do_something(test_actor_2) ...");
        Ok(())
    }
    pub async fn do_something_on_its_own(&self) -> Result<(), TestActor1Error> {
        println!("test_actor_1.do_something_on_its_own() ...");
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("TestActor1Error")]
pub struct TestActor1Error;
