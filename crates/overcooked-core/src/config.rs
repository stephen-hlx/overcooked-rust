use std::collections::HashMap;

use crate::{action::ActionDefinition, actor};

pub type ActorActionMap = HashMap<actor::Id, ActionDefinition>;
