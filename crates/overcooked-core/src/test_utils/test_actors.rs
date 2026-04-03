use std::sync::Arc;

use crate::{
    actor::{ActorBase, actor_state::ActorState},
    impl_actor_base, impl_actor_state,
    test_utils::data_store::DataStore,
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct TestActor1State {
    pub value: u8,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct TestActor2State {
    pub value: u8,
}

#[derive(Debug, PartialEq)]
pub struct TestActor1 {
    pub value: DataStore,
}

#[derive(Debug, PartialEq)]
pub struct TestActor2 {
    pub value: DataStore,
}

impl_actor_base!(TestActor1);
impl_actor_base!(TestActor2);

impl_actor_state!(TestActor1State);
impl_actor_state!(TestActor2State);

impl TestActor1 {
    pub fn new(value: u8) -> Self {
        Self {
            value: DataStore::new(value),
        }
    }

    pub fn get_value(&self) -> u8 {
        self.value.get_value()
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
        self.value.increase(1).await.map_err(|_| TestActor1Error)?;

        Ok(())
    }
}

impl TestActor2 {
    pub fn new(value: u8) -> Self {
        Self {
            value: DataStore::new(value),
        }
    }

    pub fn get_value(&self) -> u8 {
        self.value.get_value()
    }

    pub async fn decrease_inner_value_by_one(&self) -> Result<(), TestActor2Error> {
        self.value.decrease(1).await.map_err(|_| TestActor2Error)?;

        Ok(())
    }
}
#[derive(Debug, thiserror::Error)]
#[error("TestActor1Error")]
pub struct TestActor1Error;

#[derive(Debug, thiserror::Error)]
#[error("TestActor2Error")]
pub struct TestActor2Error;
