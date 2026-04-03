use std::{collections::HashMap, sync::Arc};

use crate::{
    action::{
        ActionExecutor, ActionTemplate, ActionTemplateExecutor, ExecutionResult,
        action_executor::SimpleActionExecutor,
    },
    actor::{self, actor_factory::ActorFactory, actor_state_extractor::ActorStateExtractor},
    global_state::GlobalState,
};

pub(super) struct SimpleActionTemplateExecutor<AE = SimpleActionExecutor>
where
    AE: ActionExecutor,
{
    pub action_executor: AE,
    pub actor_state_extractors: HashMap<actor::Id, Arc<dyn ActorStateExtractor>>,
    pub actor_factories: HashMap<actor::Id, Arc<dyn ActorFactory>>,
}

#[async_trait::async_trait]
impl<AE> ActionTemplateExecutor for SimpleActionTemplateExecutor<AE>
where
    AE: ActionExecutor,
{
    async fn execute(
        &self,
        template: ActionTemplate,
        global_state: GlobalState,
    ) -> ExecutionResult {
        todo!()
    }
}
