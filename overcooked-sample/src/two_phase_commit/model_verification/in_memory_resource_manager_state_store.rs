use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::two_phase_commit::model::resource_manager::{
    ResourceManagerState, ResourceManagerStateStore,
};

struct InMemoryResourceManagerStateStore {
    pub state: Arc<RwLock<ResourceManagerState>>,
}

#[async_trait]
impl ResourceManagerStateStore for InMemoryResourceManagerStateStore {
    async fn get(&self) -> ResourceManagerState {
        self.state.read().await.clone()
    }
    async fn save(&self, state: ResourceManagerState) {
        *self.state.write().await = state;
    }
}
