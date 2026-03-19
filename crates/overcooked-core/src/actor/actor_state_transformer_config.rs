use std::{collections::HashMap, sync::Arc};

use crate::actor::{self, actor_factory::ActorFactory, actor_state_extractor::ActorStateExtractor};

pub struct ActorStateTransformerConfig {
    pub actor_state_extractors: HashMap<actor::Id, Arc<dyn ActorStateExtractor>>,
    pub actor_factories: HashMap<actor::Id, Arc<dyn ActorFactory>>,
}
