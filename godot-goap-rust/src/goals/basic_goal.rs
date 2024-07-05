use crate::goals::goal_types::GoalBehaviour;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BasicGoal;

impl GoalBehaviour for BasicGoal {}
