use std::collections::HashSet;

use crate::{
    action::{ActionResult, ActionTemplate},
    global_state::GlobalState,
    transition::Transition,
};

pub struct ExecutionContext {
    global_states: HashSet<GlobalState>,
    transitions: HashSet<Transition>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            global_states: HashSet::new(),
            transitions: HashSet::new(),
        }
    }

    pub fn capture(
        &mut self,
        from: GlobalState,
        to: GlobalState,
        action_template: ActionTemplate,
        action_result: ActionResult,
    ) {
    }
}
