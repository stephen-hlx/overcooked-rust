use std::collections::HashSet;

use crate::global_state::GlobalState;

pub struct ExecutionContext {
    global_states: HashSet<GlobalState>,
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashSet};

    use crate::{
        actor::{self, local_state::LocalState},
        execution_context::ExecutionContext,
        global_state::GlobalState,
        test_utils::test_actor_states::MyActorState,
    };

    #[test]
    fn can_be_constructed() {
        let _ = ExecutionContext {
            global_states: HashSet::from([GlobalState::new(&BTreeMap::from([(
                actor::Id("some id".to_string()),
                LocalState {
                    actor_state: Box::new(MyActorState { value: 1 }),
                },
            )]))]),
        };
    }
}
