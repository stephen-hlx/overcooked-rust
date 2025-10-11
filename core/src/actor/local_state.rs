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

impl Clone for LocalState {
    fn clone(&self) -> Self {
        Self {
            actor_state: dyn_clone::clone_box(&*self.actor_state),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem;

    use crate::actor::{actor_state::ActorState, local_state::LocalState};

    #[derive(Debug, Clone, PartialEq)]
    struct MyActorState {
        values: Vec<u8>,
    }

    impl ActorState for MyActorState {}

    #[test]
    fn local_state_can_be_compared() {
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
}
