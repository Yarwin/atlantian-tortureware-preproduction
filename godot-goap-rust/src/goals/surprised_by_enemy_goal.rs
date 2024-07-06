use crate::goals::goal_types::GoalBehaviour;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BeSurprisedByEnemyGoal;

impl GoalBehaviour for BeSurprisedByEnemyGoal {}
