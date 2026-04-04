use std::{collections::BTreeMap, sync::atomic::AtomicU64};

use crate::actor::{self, local_state::LocalState};

const SEED: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocalStates(pub BTreeMap<actor::Id, LocalState>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalState {
    id: u64,
    local_states: LocalStates,
}

impl GlobalState {
    pub fn new(local_states: &LocalStates) -> Self {
        Self {
            id: SEED.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            local_states: local_states.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, sync::Arc};

    use crate::{
        actor::{self, local_state::LocalState},
        global_state::{GlobalState, LocalStates},
        test_utils::test_actors::TestActor1State,
    };

    #[test]
    fn can_be_constructed() {
        let _ = GlobalState::new(&LocalStates(BTreeMap::from([(
            actor::Id("some id".to_string()),
            LocalState {
                actor_state: Arc::new(TestActor1State { value: 1 }),
            },
        )])));
    }
}
