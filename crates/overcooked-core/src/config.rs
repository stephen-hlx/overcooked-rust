use std::collections::HashMap;

use crate::{action::LabelledAction, actor};

pub type ActorActionMap = HashMap<actor::Id, LabelledAction>;
