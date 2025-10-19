use crate::actor::actor_state::ActorState;

#[derive(Debug)]
pub struct LocalState {
    actor_state: Box<dyn ActorState>,
}

impl PartialEq for LocalState {
    fn eq(&self, other: &Self) -> bool {
        self.actor_state.dyn_eq(other.actor_state.as_ref())
    }
}

impl Eq for LocalState {}

impl Clone for LocalState {
    fn clone(&self) -> Self {
        Self {
            actor_state: dyn_clone::clone_box(&*self.actor_state),
        }
    }
}

impl PartialOrd for LocalState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.actor_state.dyn_partial_cmp(other.actor_state.as_ref())
    }
}

impl Ord for LocalState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.actor_state.dyn_cmp(other.actor_state.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use std::mem;
    use std::{cmp::Ordering, collections::BTreeSet};

    use crate::actor::{actor_state::ActorState, local_state::LocalState};

    #[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
    struct MyActorState {
        values: Vec<u8>,
    }

    impl ActorState for MyActorState {}

    #[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
    struct MyActorState2 {
        values: Vec<u8>,
    }

    impl ActorState for MyActorState2 {}

    #[test]
    fn local_state_can_be_compared_by_its_partial_order() {
        let state_1 = LocalState {
            actor_state: Box::new(MyActorState { values: vec![1] }),
        };
        let state_2 = LocalState {
            actor_state: Box::new(MyActorState { values: vec![2] }),
        };

        assert_eq!(state_1.partial_cmp(&state_1), Some(Ordering::Equal));
        assert_eq!(state_1.partial_cmp(&state_2), Some(Ordering::Less));
        assert_eq!(state_2.partial_cmp(&state_1), Some(Ordering::Greater));
        assert_eq!(
            state_1.partial_cmp(&LocalState {
                actor_state: Box::new(MyActorState2 { values: vec![1] })
            }),
            None
        );
    }

    #[test]
    fn local_state_can_be_compared_by_its_order() {
        let state_1 = LocalState {
            actor_state: Box::new(MyActorState { values: vec![1] }),
        };
        let state_2 = LocalState {
            actor_state: Box::new(MyActorState { values: vec![2] }),
        };

        assert_eq!(state_1.cmp(&state_1), Ordering::Equal);
        assert_eq!(state_1.cmp(&state_2), Ordering::Less);
        assert_eq!(state_2.cmp(&state_1), Ordering::Greater);
    }

    #[test]
    fn local_state_can_be_compared_by_its_equivalence() {
        let state = LocalState {
            actor_state: Box::new(MyActorState { values: vec![1] }),
        };

        assert_eq!(
            state,
            LocalState {
                actor_state: Box::new(MyActorState { values: vec![1] }),
            }
        );

        assert_ne!(
            state,
            LocalState {
                actor_state: Box::new(MyActorState { values: vec![2] }),
            }
        );
    }

    #[test]
    fn local_state_can_be_cloned() {
        let mut state = LocalState {
            actor_state: Box::new(MyActorState { values: vec![1] }),
        };

        let cloned_state = state.clone();

        unsafe {
            let raw_ptr: *mut dyn ActorState = mem::transmute(&mut *state.actor_state);
            let concrete_ptr = raw_ptr as *mut MyActorState;
            let concrete_ref: &mut MyActorState = &mut *concrete_ptr;
            concrete_ref.values.push(2);
        }

        assert_eq!(
            state,
            LocalState {
                actor_state: Box::new(MyActorState { values: vec![1, 2] }),
            }
        );
        assert_eq!(
            cloned_state,
            LocalState {
                actor_state: Box::new(MyActorState { values: vec![1] }),
            }
        );
    }

    #[test]
    fn local_state_can_be_added_to_btree_set() {
        let mut set = BTreeSet::new();
        set.insert(LocalState {
            actor_state: Box::new(MyActorState { values: vec![1] }),
        });
    }
}
