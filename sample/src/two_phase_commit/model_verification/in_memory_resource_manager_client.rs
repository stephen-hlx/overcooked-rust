use std::sync::Arc;

use async_trait::async_trait;

use crate::two_phase_commit::model::resource_manager::{
    Id, ResourceManagerClient, ResourceManagerClientError, ResourceManagerState,
    ResourceManagerStateStore, STATES_ALLOWED_FOR_ABORT, STATES_ALLOWED_FOR_COMMIT,
};

struct InMemoryResourceManagerClient {
    pub id: Id,
    pub state_store: Arc<dyn ResourceManagerStateStore>,
}

#[async_trait]
impl ResourceManagerClient for InMemoryResourceManagerClient {
    fn get_id(&self) -> Result<Id, ResourceManagerClientError> {
        Ok(self.id.clone())
    }

    async fn commit(&self) -> Result<(), ResourceManagerClientError> {
        let state = self.state_store.get().await;
        if !STATES_ALLOWED_FOR_COMMIT.contains(&state) {
            return Err(ResourceManagerClientError(format!(
                "commit is not allowed when state is {state:?}"
            )));
        }

        self.state_store.save(ResourceManagerState::COMMITTED).await;
        Ok(())
    }

    async fn abort(&self) -> Result<(), ResourceManagerClientError> {
        let state = self.state_store.get().await;
        if !STATES_ALLOWED_FOR_ABORT.contains(&state) {
            return Err(ResourceManagerClientError(format!(
                "abort is not allowed when state is {state:?}"
            )));
        }

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

    use crate::two_phase_commit::{
        model::resource_manager::{
            Id, MockResourceManagerStateStore, ResourceManagerClient,
            ResourceManagerState::{self, ABORTED, COMMITTED, PREPARED, WORKING},
        },
        model_verification::in_memory_resource_manager_client::InMemoryResourceManagerClient,
    };

    lazy_static! {
        static ref RESOURCE_MANAGER_ID: Id = Id("rm_1".to_string());
    }

    #[rustfmt::skip]
    //          current state, action,         success, end state
    #[test_case(WORKING,       Action::Abort,  true,    ABORTED)]
    #[test_case(WORKING,       Action::Commit, false,   WORKING)]
    #[test_case(PREPARED,      Action::Abort,  true,    ABORTED)]
    #[test_case(PREPARED,      Action::Commit, true,    COMMITTED)]
    #[test_case(COMMITTED,     Action::Abort,  false,   COMMITTED)]
    #[test_case(COMMITTED,     Action::Commit, true,    COMMITTED)]
    #[test_case(ABORTED,       Action::Abort,  true,    ABORTED)]
    #[test_case(ABORTED,       Action::Commit, false,   ABORTED)]
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

        if success {
            mock_state_store
                .expect_save()
                .with(eq(end_state))
                .once()
                .return_once(|_| ());
        }

        let resource_manager = InMemoryResourceManagerClient {
            id: RESOURCE_MANAGER_ID.clone(),
            state_store: Arc::new(mock_state_store),
        };

        assert_eq!(
            match action {
                Action::Commit => {
                    resource_manager.commit().await
                }
                Action::Abort => resource_manager.abort().await,
            }
            .is_ok(),
            success
        );
    }

    enum Action {
        Abort,
        Commit,
    }
}
