use std::sync::Arc;

use async_trait::async_trait;

use crate::two_phase_commit::model::{
    resource_manager::{
        Id, ResourceManager, ResourceManagerError, ResourceManagerStateStore,
        STATES_ALLOWED_FOR_PREPARE, STATES_ALLOWED_FOR_SELF_ABORT,
    },
    transaction_manager::TransactionManagerClient,
};

use super::ResourceManagerState;

pub struct SimpleResourceManager {
    pub id: Id,
    pub state_store: Box<dyn ResourceManagerStateStore>,
}

#[async_trait]
impl ResourceManager for SimpleResourceManager {
    async fn prepare(
        &self,
        transaction_manager: Arc<dyn TransactionManagerClient>,
    ) -> Result<(), ResourceManagerError> {
        let state = self.state_store.get().await;
        if !STATES_ALLOWED_FOR_PREPARE.contains(&state) {
            return Err(ResourceManagerError(format!(
                "prepare is not allowed when state is {state:?}"
            )));
        }

        transaction_manager.prepare(&self.id).await?;
        self.state_store.save(ResourceManagerState::PREPARED).await;
        Ok(())
    }

    async fn abort(
        &self,
        transaction_manager: Arc<dyn TransactionManagerClient>,
    ) -> Result<(), ResourceManagerError> {
        let state = self.state_store.get().await;
        if !STATES_ALLOWED_FOR_SELF_ABORT.contains(&state) {
            return Err(ResourceManagerError(format!(
                "abort is not allowed when state is {state:?}"
            )));
        }

        transaction_manager.abort(&self.id).await?;
        self.state_store.save(ResourceManagerState::ABORTED).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use lazy_static::lazy_static;
    use mockall::predicate::eq;
    use test_case::test_case;

    use crate::two_phase_commit::model::{
        resource_manager::{
            Id, MockResourceManagerStateStore, ResourceManager,
            ResourceManagerState::{self, ABORTED, COMMITTED, PREPARED, WORKING},
            simple_resource_manager::SimpleResourceManager,
        },
        transaction_manager::MockTransactionManagerClient,
    };

    lazy_static! {
        static ref RESOURCE_MANAGER_ID: Id = Id("rm_1".to_string());
    }

    #[rustfmt::skip]
    //          current state, action,          success, end state
    #[test_case(WORKING,       Action::Abort,   true,    ABORTED)]
    #[test_case(WORKING,       Action::Prepare, true,    PREPARED)]
    #[test_case(PREPARED,      Action::Abort,   false,   PREPARED)]
    #[test_case(PREPARED,      Action::Prepare, true,    PREPARED)]
    #[test_case(COMMITTED,     Action::Abort,   false,   COMMITTED)]
    #[test_case(COMMITTED,     Action::Prepare, false,   COMMITTED)]
    #[test_case(ABORTED,       Action::Abort,   false,   ABORTED)]
    #[test_case(ABORTED,       Action::Prepare, false,   ABORTED)]
    #[tokio::test]
    async fn works(
        current_state: ResourceManagerState,
        action: Action,
        success: bool,
        end_state: ResourceManagerState,
    ) {
        if !success {
            assert_eq!(current_state, end_state);
        }

        let mut mock_state_store = MockResourceManagerStateStore::new();
        mock_state_store
            .expect_get()
            .once()
            .return_once(|| current_state);

        let mut mock_transaction_manager_client = MockTransactionManagerClient::new();

        if success {
            mock_state_store
                .expect_save()
                .with(eq(end_state))
                .once()
                .return_once(|_| ());

            match action {
                Action::Prepare => {
                    mock_transaction_manager_client
                        .expect_prepare()
                        .with(eq(RESOURCE_MANAGER_ID.clone()))
                        .once()
                        .return_once(|_| Ok(()));
                }
                Action::Abort => {
                    mock_transaction_manager_client
                        .expect_abort()
                        .with(eq(RESOURCE_MANAGER_ID.clone()))
                        .once()
                        .return_once(|_| Ok(()));
                }
            };
        }

        let resource_manager = SimpleResourceManager {
            id: RESOURCE_MANAGER_ID.clone(),
            state_store: Box::new(mock_state_store),
        };

        assert_eq!(
            match action {
                Action::Prepare => {
                    resource_manager
                        .prepare(Arc::new(mock_transaction_manager_client))
                        .await
                }
                Action::Abort =>
                    resource_manager
                        .abort(Arc::new(mock_transaction_manager_client))
                        .await,
            }
            .is_ok(),
            success
        );
    }

    enum Action {
        Prepare,
        Abort,
    }
}
