use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::two_phase_commit::model::{
    resource_manager::{Id, ResourceManagerClient, ResourceManagerState},
    transaction_manager::{
        STATES_ALLOWED_FOR_ABORT, STATES_ALLOWED_FOR_COMMIT, TransactionManager,
        TransactionManagerError,
    },
};

#[derive(Debug)]
pub struct SimpleTransactionManager {
    pub resource_manager_states: RwLock<HashMap<Id, ResourceManagerState>>,
}

#[async_trait]
impl TransactionManager for SimpleTransactionManager {
    async fn abort(
        &self,
        resource_manager: Arc<dyn ResourceManagerClient>,
    ) -> Result<(), TransactionManagerError> {
        validate_state(
            &*self.resource_manager_states.read().await,
            &STATES_ALLOWED_FOR_ABORT,
        )?;
        resource_manager.abort().await?;
        self.resource_manager_states
            .write()
            .await
            .insert(resource_manager.get_id()?, ResourceManagerState::ABORTED);
        Ok(())
    }

    async fn commit(
        &self,
        resource_manager: Arc<dyn ResourceManagerClient>,
    ) -> Result<(), TransactionManagerError> {
        validate_state(
            &*self.resource_manager_states.read().await,
            &STATES_ALLOWED_FOR_COMMIT,
        )?;
        resource_manager.commit().await?;
        self.resource_manager_states
            .write()
            .await
            .insert(resource_manager.get_id()?, ResourceManagerState::COMMITTED);
        Ok(())
    }
}

fn validate_state(
    states: &HashMap<Id, ResourceManagerState>,
    allowed_states: &[ResourceManagerState],
) -> Result<(), TransactionManagerError> {
    for (id, state) in states {
        if !allowed_states.contains(state) {
            return Err(TransactionManagerError(format!(
                "Current state {state:?} of ResourceManager({id}) is not allowed for the action"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use std::{collections::HashMap, sync::Arc};
    use test_case::test_case;

    use tokio::sync::RwLock;

    use crate::two_phase_commit::model::{
        resource_manager::{
            Id, MockResourceManagerClient,
            ResourceManagerState::{self, ABORTED, COMMITTED, PREPARED, WORKING},
        },
        transaction_manager::{
            TransactionManager, simple_transaction_manager::SimpleTransactionManager,
        },
    };

    lazy_static! {
        static ref RESOURCE_MANAGER_1_ID: Id = Id("rm_1".to_string());
        static ref RESOURCE_MANAGER_2_ID: Id = Id("rm_2".to_string());
    }

    #[rustfmt::skip]
    //         |<- Current States ->|                         |<- Expected States ->|
    //          RM1_state, RM2_state, action_on_RM1,  success, RM1_state, RM2_state
    #[test_case(ABORTED,   ABORTED,   Action::Commit, false,   ABORTED,   ABORTED)]
    #[test_case(ABORTED,   ABORTED,   Action::Abort,  true,    ABORTED,   ABORTED)]
    #[test_case(ABORTED,   COMMITTED, Action::Commit, false,   ABORTED,   COMMITTED)]
    #[test_case(ABORTED,   COMMITTED, Action::Abort,  false,   ABORTED,   COMMITTED)]
    #[test_case(ABORTED,   PREPARED,  Action::Commit, false,   ABORTED,   PREPARED)]
    #[test_case(ABORTED,   PREPARED,  Action::Abort,  true,    ABORTED,   PREPARED)]
    #[test_case(ABORTED,   WORKING,   Action::Commit, false,   ABORTED,   WORKING)]
    #[test_case(ABORTED,   WORKING,   Action::Abort,  true,    ABORTED,   WORKING)]
    #[test_case(COMMITTED, ABORTED,   Action::Commit, false,   COMMITTED, ABORTED)]
    #[test_case(COMMITTED, ABORTED,   Action::Abort,  false,   COMMITTED, ABORTED)]
    #[test_case(COMMITTED, COMMITTED, Action::Commit, true,    COMMITTED, COMMITTED)]
    #[test_case(COMMITTED, COMMITTED, Action::Abort,  false,   COMMITTED, COMMITTED)]
    #[test_case(COMMITTED, PREPARED,  Action::Commit, true,    COMMITTED, PREPARED)]
    #[test_case(COMMITTED, PREPARED,  Action::Abort,  false,   COMMITTED, PREPARED)]
    #[test_case(COMMITTED, WORKING,   Action::Commit, false,   COMMITTED, WORKING)]
    #[test_case(COMMITTED, WORKING,   Action::Abort,  false,   COMMITTED, WORKING)]
    #[test_case(PREPARED,  ABORTED,   Action::Commit, false,   PREPARED,  ABORTED)]
    #[test_case(PREPARED,  ABORTED,   Action::Abort,  true,    ABORTED,   ABORTED)]
    #[test_case(PREPARED,  COMMITTED, Action::Commit, true,    COMMITTED, COMMITTED)]
    #[test_case(PREPARED,  COMMITTED, Action::Abort,  false,   PREPARED,  COMMITTED)]
    #[test_case(PREPARED,  PREPARED,  Action::Commit, true,    COMMITTED, PREPARED)]
    #[test_case(PREPARED,  PREPARED,  Action::Abort,  true,    ABORTED,   PREPARED)]
    #[test_case(PREPARED,  WORKING,   Action::Commit, false,   PREPARED,  WORKING)]
    #[test_case(PREPARED,  WORKING,   Action::Abort,  true,    ABORTED,   WORKING)]
    #[test_case(WORKING,   ABORTED,   Action::Commit, false,   WORKING,   ABORTED)]
    #[test_case(WORKING,   ABORTED,   Action::Abort,  true,    ABORTED,   ABORTED)]
    #[test_case(WORKING,   COMMITTED, Action::Commit, false,   WORKING,   COMMITTED)]
    #[test_case(WORKING,   COMMITTED, Action::Abort,  false,   WORKING,   COMMITTED)]
    #[test_case(WORKING,   PREPARED,  Action::Commit, false,   WORKING,   PREPARED)]
    #[test_case(WORKING,   PREPARED,  Action::Abort,  true,    ABORTED,   PREPARED)]
    #[test_case(WORKING,   WORKING,   Action::Commit, false,   WORKING,   WORKING)]
    #[test_case(WORKING,   WORKING,   Action::Abort,  true,    ABORTED,   WORKING)]
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

        let transaction_manager = SimpleTransactionManager {
            resource_manager_states: RwLock::new(HashMap::from([
                (RESOURCE_MANAGER_1_ID.clone(), resource_manager_1_state),
                (RESOURCE_MANAGER_2_ID.clone(), resource_manager_2_state),
            ])),
        };

        let mut resource_manager_1_client = MockResourceManagerClient::new();

        if success {
            resource_manager_1_client
                .expect_get_id()
                .once()
                .return_once(|| Ok(RESOURCE_MANAGER_1_ID.clone()));

            match action {
                Action::Abort => {
                    resource_manager_1_client
                        .expect_abort()
                        .once()
                        .return_once(|| Ok(()));
                }
                Action::Commit => {
                    resource_manager_1_client
                        .expect_commit()
                        .once()
                        .return_once(|| Ok(()));
                }
            }
        }

        match action {
            Action::Abort => {
                assert_eq!(
                    transaction_manager
                        .abort(Arc::new(resource_manager_1_client))
                        .await
                        .is_ok(),
                    success
                );
            }
            Action::Commit => {
                assert_eq!(
                    transaction_manager
                        .commit(Arc::new(resource_manager_1_client))
                        .await
                        .is_ok(),
                    success
                );
            }
        }

        assert_eq!(
            *transaction_manager.resource_manager_states.read().await,
            HashMap::from([
                (RESOURCE_MANAGER_1_ID.clone(), expected_resource_manager_1_state),
                (RESOURCE_MANAGER_2_ID.clone(), expected_resource_manager_2_state),
            ]),
        );
    }

    enum Action {
        Abort,
        Commit,
    }
}
