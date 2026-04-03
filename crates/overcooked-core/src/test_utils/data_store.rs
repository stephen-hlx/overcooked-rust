use std::sync::{Arc, RwLock};

/// todo:
/// rename to data store
/// proper error
#[derive(Debug)]
pub struct DataStore {
    value: Arc<RwLock<u8>>,
}

impl DataStore {
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

#[cfg(test)]
mod tests {
    use super::DataStore;

    #[tokio::test]
    async fn updatable_value_works() {
        let updatable_value = DataStore::new(5);
        assert_eq!(updatable_value.get_value(), 5);
        updatable_value.increase(1).await.unwrap();
        assert_eq!(updatable_value.get_value(), 6);
        updatable_value.decrease(1).await.unwrap();
        assert_eq!(updatable_value.get_value(), 5);
    }
}
