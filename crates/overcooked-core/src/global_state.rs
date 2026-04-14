use std::{collections::BTreeMap, sync::atomic::AtomicU64};

use crate::actor::{self, local_state::LocalState};

const SEED: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Eq)]
pub struct GlobalState {
    id: u64,
    local_states: BTreeMap<actor::Id, LocalState>,
}

impl GlobalState {
    pub fn new(local_states: BTreeMap<actor::Id, LocalState>) -> Self {
        Self {
            id: SEED.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            local_states: local_states,
        }
    }

    pub fn get_local_state(&self, actor_id: &actor::Id) -> LocalState {
        self.local_states
            .get(actor_id)
            .expect(&format!("LocalState for {actor_id:?} not found"))
            .clone()
    }

    pub fn insert_local_state(&mut self, actor_id: actor::Id, local_state: LocalState) {
        self.local_states.insert(actor_id, local_state);
    }
}

impl PartialEq for GlobalState {
    fn eq(&self, other: &Self) -> bool {
        self.local_states == other.local_states
    }
}

impl std::hash::Hash for GlobalState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.local_states.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        hash::{DefaultHasher, Hasher},
        sync::Arc,
    };

    use crate::{
        actor::{self, local_state::LocalState},
        global_state::GlobalState,
        test_utils::test_actors::TestActor1State,
    };

    #[test]
    fn can_be_constructed() {
        let _ = GlobalState::new(create_local_states(0));
    }

    #[test]
    fn comparison_works() {
        assert_ne!(
            GlobalState {
                id: 1,
                local_states: create_local_states(0)
            },
            GlobalState {
                id: 1,
                local_states: create_local_states(1)
            }
        )
    }

    #[test]
    fn comparison_does_not_take_into_account_id() {
        assert_eq!(
            GlobalState {
                id: 1,
                local_states: create_local_states(0)
            },
            GlobalState {
                id: 2,
                local_states: create_local_states(0)
            }
        )
    }

    #[test]
    fn hash_works() {
        assert_ne!(
            hash_it(&GlobalState {
                id: 1,
                local_states: create_local_states(0)
            }),
            hash_it(&GlobalState {
                id: 1,
                local_states: create_local_states(1)
            })
        );
    }

    #[test]
    fn hash_does_not_take_into_account_id() {
        assert_eq!(
            hash_it(&GlobalState {
                id: 1,
                local_states: create_local_states(0)
            }),
            hash_it(&GlobalState {
                id: 2,
                local_states: create_local_states(0)
            })
        );
    }

    fn hash_it<T: std::hash::Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    fn create_local_states(actor_1_state_value: u8) -> BTreeMap<actor::Id, LocalState> {
        BTreeMap::from([(
            actor::Id("actor-1".to_string()),
            LocalState {
                actor_state: Arc::new(TestActor1State {
                    value: actor_1_state_value,
                }),
            },
        )])
    }
}
