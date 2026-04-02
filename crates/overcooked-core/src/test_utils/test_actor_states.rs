use std::sync::{Arc, RwLock};

use crate::{
    actor::{ActorBase, actor_state::ActorState},
    impl_actor_base,
};

pub struct UpdatableValue {
    value: Arc<RwLock<u8>>,
}

impl UpdatableValue {
    pub fn new(value: u8) -> Self {
        Self {
            value: Arc::new(RwLock::new(value)),
        }
    }

    pub fn get_value(&self) -> u8 {
        *self.value.read().unwrap()
    }

    pub async fn increase(&self, by: u8) -> Result<(), String> {
        let mut value = self.value.write().map_err(|err| err.to_string())?;
        *value = *value + by;

        Ok(())
    }

    pub async fn decrease(&self, by: u8) -> Result<(), String> {
        let mut value = self.value.write().map_err(|err| err.to_string())?;
        *value = *value - by;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct TestActor1State {
    pub value: u8,
}

impl ActorState for TestActor1State {}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct TestActorState2 {
    pub value: u8,
}

impl ActorState for TestActorState2 {}

pub struct TestActor1 {
    pub value: UpdatableValue,
}

pub struct TestActor2 {
    pub value: UpdatableValue,
}

impl_actor_base!(TestActor1);
impl_actor_base!(TestActor2);

impl TestActor1 {
    pub fn new(value: u8) -> Self {
        Self {
            value: UpdatableValue::new(value),
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
            value: UpdatableValue::new(value),
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

#[cfg(test)]
mod tests {
    use crate::test_utils::test_actor_states::UpdatableValue;

    #[tokio::test]
    async fn updatable_value_works() {
        let updatable_value = UpdatableValue::new(5);
        assert_eq!(updatable_value.get_value(), 5);
        updatable_value.increase(1).await.unwrap();
        assert_eq!(updatable_value.get_value(), 6);
        updatable_value.decrease(1).await.unwrap();
        assert_eq!(updatable_value.get_value(), 5);
    }
}
