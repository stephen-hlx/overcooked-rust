use std::collections::HashMap;

use async_trait::async_trait;

use tokio::sync::RwLock;

use crate::two_phase_commit::model::{
    resource_manager::{Id, ResourceManagerState},
    transaction_manager::{
        STATES_ALLOWED_FOR_PREPARE, STATES_ALLOWED_FOR_SELF_ABORT, TransactionManagerClient,
        TransactionManagerClientError,
    },
};

struct InMemoryTransactionManagerClient {
    pub resource_manager_states: RwLock<HashMap<Id, ResourceManagerState>>,
}

#[async_trait]
impl TransactionManagerClient for InMemoryTransactionManagerClient {
    async fn prepare(&self, id: &Id) -> Result<(), TransactionManagerClientError> {
        self.validate_state(id, &STATES_ALLOWED_FOR_PREPARE).await?;

        self.resource_manager_states
            .write()
            .await
            .insert(id.clone(), ResourceManagerState::PREPARED);
        Ok(())
    }

    async fn abort(&self, id: &Id) -> Result<(), TransactionManagerClientError> {
        self.validate_state(id, &STATES_ALLOWED_FOR_SELF_ABORT)
            .await?;

        self.resource_manager_states
            .write()
            .await
            .insert(id.clone(), ResourceManagerState::ABORTED);
        Ok(())
    }
}

impl InMemoryTransactionManagerClient {
    async fn validate_state(
        &self,
        id: &Id,
        allowed_states: &[ResourceManagerState],
    ) -> Result<(), TransactionManagerClientError> {
        let states = self.resource_manager_states.read().await;

        let commit_has_started = states
            .iter()
            .any(|(_, state)| *state == ResourceManagerState::COMMITTED);

        if commit_has_started {
            return Err(TransactionManagerClientError(
                "Commit has started, action is not allowed".to_string(),
            ));
        }

        let state = match states.get(&id) {
            Some(state) => state,
            None => {
                return Err(TransactionManagerClientError(format!("Unknown id: {id}")));
            }
        };

        if !allowed_states.contains(state) {
            return Err(TransactionManagerClientError(format!(
                "Action is not allowed for current state {state:?} for ResourceManager({id})"
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use std::collections::HashMap;
    use test_case::test_case;

    use tokio::sync::RwLock;

    use crate::two_phase_commit::{
        model::{
            resource_manager::{
                Id,
                ResourceManagerState::{self, ABORTED, COMMITTED, PREPARED, WORKING},
            },
            transaction_manager::TransactionManagerClient,
        },
        model_verification::in_memory_transaction_manager_client::InMemoryTransactionManagerClient,
    };

    lazy_static! {
        static ref RESOURCE_MANAGER_1_ID: Id = Id("rm_1".to_string());
        static ref RESOURCE_MANAGER_2_ID: Id = Id("rm_2".to_string());
    }

    #[rustfmt::skip]
    //         |<- Current States ->|                          |<- Expected States ->|
    //          RM1_state, RM2_state, action_on_RM1,   success, RM1_state, RM2_state
    #[test_case(ABORTED,   ABORTED,   Action::Prepare, false,   ABORTED,   ABORTED)]
    #[test_case(ABORTED,   ABORTED,   Action::Abort,   true,    ABORTED,   ABORTED)]
    #[test_case(ABORTED,   COMMITTED, Action::Prepare, false,   ABORTED,   COMMITTED)]
    #[test_case(ABORTED,   COMMITTED, Action::Abort,   false,   ABORTED,   COMMITTED)]
    #[test_case(ABORTED,   PREPARED,  Action::Prepare, false,   ABORTED,   PREPARED)]
    #[test_case(ABORTED,   PREPARED,  Action::Abort,   true,    ABORTED,   PREPARED)]
    #[test_case(ABORTED,   WORKING,   Action::Prepare, false,   ABORTED,   WORKING)]
    #[test_case(ABORTED,   WORKING,   Action::Abort,   true,    ABORTED,   WORKING)]
    #[test_case(COMMITTED, ABORTED,   Action::Prepare, false,   COMMITTED, ABORTED)]
    #[test_case(COMMITTED, ABORTED,   Action::Abort,   false,   COMMITTED, ABORTED)]
    #[test_case(COMMITTED, COMMITTED, Action::Prepare, false,   COMMITTED, COMMITTED)]
    #[test_case(COMMITTED, COMMITTED, Action::Abort,   false,   COMMITTED, COMMITTED)]
    #[test_case(COMMITTED, PREPARED,  Action::Prepare, false,   COMMITTED, PREPARED)]
    #[test_case(COMMITTED, PREPARED,  Action::Abort,   false,   COMMITTED, PREPARED)]
    #[test_case(COMMITTED, WORKING,   Action::Prepare, false,   COMMITTED, WORKING)]
    #[test_case(COMMITTED, WORKING,   Action::Abort,   false,   COMMITTED, WORKING)]
    #[test_case(PREPARED,  ABORTED,   Action::Prepare, true,    PREPARED,  ABORTED)]
    #[test_case(PREPARED,  ABORTED,   Action::Abort,   false,   PREPARED,  ABORTED)]
    #[test_case(PREPARED,  COMMITTED, Action::Prepare, false,   PREPARED,  COMMITTED)]
    #[test_case(PREPARED,  COMMITTED, Action::Abort,   false,   PREPARED,  COMMITTED)]
    #[test_case(PREPARED,  PREPARED,  Action::Prepare, true,    PREPARED,  PREPARED)]
    #[test_case(PREPARED,  PREPARED,  Action::Abort,   false,   PREPARED,  PREPARED)]
    #[test_case(PREPARED,  WORKING,   Action::Prepare, true,    PREPARED,  WORKING)]
    #[test_case(PREPARED,  WORKING,   Action::Abort,   false,   PREPARED,  WORKING)]
    #[test_case(WORKING,   ABORTED,   Action::Prepare, true,    PREPARED,  ABORTED)]
    #[test_case(WORKING,   ABORTED,   Action::Abort,   true,    ABORTED,   ABORTED)]
    #[test_case(WORKING,   COMMITTED, Action::Prepare, false,   WORKING,   COMMITTED)]
    #[test_case(WORKING,   COMMITTED, Action::Abort,   false,   WORKING,   COMMITTED)]
    #[test_case(WORKING,   PREPARED,  Action::Prepare, true,    PREPARED,  PREPARED)]
    #[test_case(WORKING,   PREPARED,  Action::Abort,   true,    ABORTED,   PREPARED)]
    #[test_case(WORKING,   WORKING,   Action::Prepare, true,    PREPARED,  WORKING)]
    #[test_case(WORKING,   WORKING,   Action::Abort,   true,    ABORTED,   WORKING)]
    #[tokio::test]
    async fn works(
        resource_manager_1_state: ResourceManagerState,
        resource_manager_2_state: ResourceManagerState,
        action: Action,
        success: bool,
        expected_resource_manager_1_state: ResourceManagerState,
        expected_resource_manager_2_state: ResourceManagerState,
    ) {
        if !success {
            assert_eq!(resource_manager_1_state, expected_resource_manager_1_state);
            assert_eq!(resource_manager_2_state, expected_resource_manager_2_state);
        }

        let client = InMemoryTransactionManagerClient {
            resource_manager_states: RwLock::new(HashMap::from([
                (RESOURCE_MANAGER_1_ID.clone(), resource_manager_1_state),
                (RESOURCE_MANAGER_2_ID.clone(), resource_manager_2_state),
            ])),
        };

        match action {
            Action::Abort => assert_eq!(client.abort(&RESOURCE_MANAGER_1_ID).await.is_ok(), success),
            Action::Prepare => {
                assert_eq!(client.prepare(&RESOURCE_MANAGER_1_ID).await.is_ok(), success)
            }
        }

        assert_eq!(
            *client.resource_manager_states.read().await,
            HashMap::from([
                (RESOURCE_MANAGER_1_ID.clone(), expected_resource_manager_1_state),
                (RESOURCE_MANAGER_2_ID.clone(), expected_resource_manager_2_state),
            ]),
        );
    }

    enum Action {
        Abort,
        Prepare,
    }
}
