use crate::{
    action::{ActionResult, ActionTemplate},
    global_state::GlobalState,
};

#[derive(Debug, Clone)]
pub struct Transition {
    pub from: GlobalState,
    pub to: GlobalState,
    pub action_template: ActionTemplate,
    pub action_result: ActionResult,
}

impl std::hash::Hash for Transition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
        self.action_template.hash(state);
    }
}

impl PartialEq for Transition {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from
            && self.to == other.to
            && self.action_template == other.action_template
    }
}

impl Eq for Transition {}
