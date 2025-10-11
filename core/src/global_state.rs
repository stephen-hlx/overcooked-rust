use std::{collections::HashMap, sync::atomic::AtomicU64};

use crate::actor::{self, local_state::LocalState};

const SEED: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalState {
    id: u64,
    local_states: HashMap<actor::Id, LocalState>,
}

impl GlobalState {
    pub fn new(local_states: &HashMap<actor::Id, LocalState>) -> Self {
        Self {
            id: SEED.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            local_states: local_states.clone(),
        }
    }
}
