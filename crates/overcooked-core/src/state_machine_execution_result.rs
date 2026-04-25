use std::collections::HashSet;

use crate::{
    action::{ActionResult, ActionTemplate},
    global_state::GlobalState,
    transition::Transition,
};

#[derive(Debug, PartialEq, Eq)]
pub struct StateMachineExecutionResult {
    pub transitions: HashSet<Transition>,
    pub invariant_violations: HashSet<GlobalState>,
}
