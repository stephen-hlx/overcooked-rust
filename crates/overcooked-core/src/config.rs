use std::collections::HashMap;

use crate::{action::ActionTemplate, actor};

pub type ActorActionMap = HashMap<actor::Id, ActionTemplate>;
