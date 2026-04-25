use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

use crate::{
    actor::{ActorBase, actor_state::ActorState},
    impl_actor_base, impl_actor_state, intransitive_action, transitive_action,
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct TestActor1State {
    pub value: u8,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct TestActor2State {
    pub value: u8,
}

#[derive(Debug)]
pub struct TestActor1 {
    pub value: AtomicU8,
}

#[derive(Debug)]
pub struct TestActor2 {
    pub value: AtomicU8,
}

impl_actor_base!(TestActor1);
impl_actor_base!(TestActor2);

impl_actor_state!(TestActor1State);
impl_actor_state!(TestActor2State);

intransitive_action!(TestActor1, increase_inner_value_by_one);
transitive_action!(TestActor1, decrease_test_actor_2_value_by_one, TestActor2);

impl TestActor1 {
    pub fn new(value: u8) -> Self {
        Self {
            value: AtomicU8::new(value),
        }
    }

    pub fn get_value(&self) -> u8 {
        self.value.load(Ordering::Relaxed)
    }

    pub async fn decrease_test_actor_2_value_by_one(
        &self,
        test_actor_2: &TestActor2,
    ) -> Result<(), TestActor1Error> {
        test_actor_2
            .decrease_inner_value_by_one()
            .await
            .map_err(|_| TestActor1Error)?;

        Ok(())
    }

    pub async fn increase_inner_value_by_one(&self) -> Result<(), TestActor1Error> {
        self.value.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }
}

impl PartialEq for TestActor1 {
    fn eq(&self, other: &Self) -> bool {
        self.value.load(Ordering::Relaxed) == other.value.load(Ordering::Relaxed)
    }
}

impl TestActor2 {
    pub fn new(value: u8) -> Self {
        Self {
            value: AtomicU8::new(value),
        }
    }

    pub fn get_value(&self) -> u8 {
        self.value.load(Ordering::Relaxed)
    }

    pub async fn decrease_inner_value_by_one(&self) -> Result<(), TestActor2Error> {
        self.value.fetch_sub(1, Ordering::Relaxed);

        Ok(())
    }
}

impl PartialEq for TestActor2 {
    fn eq(&self, other: &Self) -> bool {
        self.value.load(Ordering::Relaxed) == other.value.load(Ordering::Relaxed)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("TestActor1Error")]
pub struct TestActor1Error;

#[derive(Debug, thiserror::Error)]
#[error("TestActor2Error")]
pub struct TestActor2Error;
