use std::collections::HashSet;

use crate::global_state::GlobalState;

pub struct ExecutionContext {
    global_states: HashSet<GlobalState>,
}
