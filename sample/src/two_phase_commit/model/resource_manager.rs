use std::{fmt::Display, sync::Arc};

use async_trait::async_trait;

use mockall::automock;

use crate::two_phase_commit::model::transaction_manager::{
    TransactionManagerClient, TransactionManagerClientError,
};

mod simple_resource_manager;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Id(pub String);

#[async_trait]
trait ResourceManager {
    async fn prepare(
        &self,
        transaction_manager: Arc<dyn TransactionManagerClient>,
    ) -> Result<(), ResourceManagerError>;
    async fn abort(
        &self,
        transaction_manager: Arc<dyn TransactionManagerClient>,
    ) -> Result<(), ResourceManagerError>;
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[automock]
#[async_trait]
pub trait ResourceManagerStateStore: Send + Sync {
    async fn get(&self) -> ResourceManagerState;
    async fn save(&self, state: ResourceManagerState);
}

#[automock]
#[async_trait]
pub trait ResourceManagerClient: Send + Sync {
    fn get_id(&self) -> Result<Id, ResourceManagerClientError>;
    async fn commit(&self) -> Result<(), ResourceManagerClientError>;
    async fn abort(&self) -> Result<(), ResourceManagerClientError>;
}

#[derive(Debug, thiserror::Error)]
#[error("ResourceManagerError {0}")]
pub struct ResourceManagerError(String);

#[derive(Debug, thiserror::Error)]
#[error("ResourceManagerClientError {0}")]
pub struct ResourceManagerClientError(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum ResourceManagerState {
    WORKING,
    PREPARED,
    COMMITTED,
    ABORTED,
}

impl From<TransactionManagerClientError> for ResourceManagerError {
    fn from(value: TransactionManagerClientError) -> Self {
        Self(value.to_string())
    }
}

const STATES_ALLOWED_FOR_PREPARE: [ResourceManagerState; 2] = [
    ResourceManagerState::WORKING,
    ResourceManagerState::PREPARED,
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

const STATES_ALLOWED_FOR_SELF_ABORT: [ResourceManagerState; 1] = [ResourceManagerState::WORKING];
