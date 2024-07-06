use crate::goals::goal_types::GoalBehaviour;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DodgeGoal;

impl GoalBehaviour for DodgeGoal {}
