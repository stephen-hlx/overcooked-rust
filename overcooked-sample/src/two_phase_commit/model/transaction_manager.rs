use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;

use crate::two_phase_commit::model::resource_manager::{
    Id, ResourceManagerClient, ResourceManagerClientError, ResourceManagerState,
};

mod simple_transaction_manager;

#[async_trait]
trait TransactionManager {
    async fn abort(
        &self,
        resource_manager: Arc<dyn ResourceManagerClient>,
    ) -> Result<(), TransactionManagerError>;
    async fn commit(
        &self,
        resource_manager: Arc<dyn ResourceManagerClient>,
    ) -> Result<(), TransactionManagerError>;
}

#[automock]
#[async_trait]
pub trait TransactionManagerClient: Send + Sync {
    async fn prepare(&self, id: &Id) -> Result<(), TransactionManagerClientError>;
    async fn abort(&self, id: &Id) -> Result<(), TransactionManagerClientError>;
}

#[derive(Debug, thiserror::Error)]
#[error("TransactionManagerError {0}")]
struct TransactionManagerError(String);

#[derive(Debug, thiserror::Error)]
#[error("TransactionManagerClientError {0}")]
pub struct TransactionManagerClientError(pub String);

impl From<ResourceManagerClientError> for TransactionManagerError {
    fn from(value: ResourceManagerClientError) -> Self {
        Self(value.to_string())
    }
}

pub const STATES_ALLOWED_FOR_PREPARE: [ResourceManagerState; 2] = [
    ResourceManagerState::PREPARED,
    ResourceManagerState::WORKING,
];

pub const STATES_ALLOWED_FOR_COMMIT: [ResourceManagerState; 2] = [
    ResourceManagerState::COMMITTED,
    ResourceManagerState::PREPARED,
];

pub const STATES_ALLOWED_FOR_ABORT: [ResourceManagerState; 3] = [
    ResourceManagerState::WORKING,
    ResourceManagerState::ABORTED,
    ResourceManagerState::PREPARED,
];

pub const STATES_ALLOWED_FOR_SELF_ABORT: [ResourceManagerState; 2] =
    [ResourceManagerState::WORKING, ResourceManagerState::ABORTED];
