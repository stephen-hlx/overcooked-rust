use std::collections::HashSet;

use crate::global_state::GlobalState;

pub struct ExecutionContext {
    global_states: HashSet<GlobalState>,
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, HashSet},
        sync::Arc,
    };

    use crate::{
        actor::{self, local_state::LocalState},
        execution_context::ExecutionContext,
        global_state::{GlobalState, LocalStates},
        test_utils::test_actors::TestActor1State,
    };

    #[test]
    fn can_be_constructed() {
        let _ = ExecutionContext {
            global_states: HashSet::from([GlobalState::new(&LocalStates(BTreeMap::from([(
                actor::Id("some id".to_string()),
                LocalState {
                    actor_state: Arc::new(TestActor1State { value: 1 }),
                },
            )])))]),
        };
    }
}
