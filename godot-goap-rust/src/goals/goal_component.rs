use crate::ai::world_state::{NodeTypeEnum, WorldState};
use crate::goals::goal_types::GoalType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GoalComponent {
    pub name: String,
    pub priority: u32,
    pub goto_target: Option<NodeTypeEnum>,
    pub goal_type: GoalType,
    pub desired_state: WorldState,
    pub required_state: WorldState,
}
