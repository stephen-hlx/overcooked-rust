use std::{collections::HashMap, sync::Arc};

use crate::{
    action::{
        ActionResult, IntransitiveAction, TransitiveAction,
        action_template_executor::action_executor::SimpleActionExecutor,
    },
    actor::{
        self, ActorBase, actor_factory::ActorFactory, actor_state_extractor::ActorStateExtractor,
    },
};

mod action;
mod action_executor;

#[mockall::automock]
#[async_trait::async_trait]
trait ActionExecutor {
    async fn execute(&self, action: Action) -> ActionResult;
}

enum Action {
    Intransitive {
        performer: Arc<dyn ActorBase>,
        action: IntransitiveAction,
    },
    Transitive {
        performer: Arc<dyn ActorBase>,
        receiver: Arc<dyn ActorBase>,
        action: TransitiveAction,
    },
}

pub(super) struct SimpleActionTemplateExecutor<AE = SimpleActionExecutor>
where
    AE: ActionExecutor,
{
    pub action_executor: AE,
    pub actor_state_extractors: HashMap<actor::Id, Box<dyn ActorStateExtractor>>,
    pub actor_factories: HashMap<actor::Id, Box<dyn ActorFactory>>,
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, HashMap},
        sync::{Arc, LazyLock},
    };

    use crate::{
        action::action_template_executor::{MockActionExecutor, SimpleActionTemplateExecutor},
        actor::{
            self,
            actor_factory::{ActorFactory, MockActorFactory},
            actor_state_extractor::{ActorStateExtractor, MockActorStateExtractor},
            local_state::LocalState,
        },
        global_state::{GlobalState, LocalStates},
        test_utils::test_actors::{TestActor1, TestActor1State},
    };

    static ACTOR_1_ID: LazyLock<actor::Id> = LazyLock::new(|| actor::Id("actor_1".to_string()));
    static ACTOR_2_ID: LazyLock<actor::Id> = LazyLock::new(|| actor::Id("actor_2".to_string()));

    #[tokio::test]
    async fn intransitive_action_template_execution_works() {
        let actor = Arc::new(TestActor1::new(10));
        let local_states = LocalStates(BTreeMap::from([(
            ACTOR_1_ID.clone(),
            LocalState {
                actor_state: Box::new(TestActor1State { value: 10 }),
            },
        )]));

        let mut action_executor = MockActionExecutor::new();
        let mut actor_1_factor = MockActorFactory::new();
        let mut actor_1_state_extractor = MockActorStateExtractor::new();

        let executor = SimpleActionTemplateExecutor {
            action_executor,
            actor_state_extractors: HashMap::from([(
                ACTOR_1_ID.clone(),
                Box::new(actor_1_state_extractor) as Box<dyn ActorStateExtractor>,
            )]),
            actor_factories: HashMap::from([(
                ACTOR_1_ID.clone(),
                Box::new(actor_1_factor) as Box<dyn ActorFactory>,
            )]),
        };
    }
}
