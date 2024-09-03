use crate::ai::world_state::{NodeTypeEnum, WorldState};
use crate::goap_goals::goal_types::GoalType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GoalComponent {
    pub name: String,
    pub priority: u32,
    #[serde(default = "is_interruptible_default")]
    pub is_interruptible: bool,
    pub goto_target: Option<NodeTypeEnum>,
    pub goal_type: GoalType,
    pub desired_state: WorldState,
    pub required_state: WorldState,
}

fn is_interruptible_default() -> bool {true}
