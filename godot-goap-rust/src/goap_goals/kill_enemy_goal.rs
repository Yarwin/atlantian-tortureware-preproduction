use crate::ai::world_state::{WSProperty, WorldStateProperty};
use crate::goap_goals::goal_component::GoalComponent;
use crate::goap_goals::goal_types::{AgentGoalWorldContext, GoalBehaviour};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct KillEnemyGoal;

impl GoalBehaviour for KillEnemyGoal {
    fn deactivate(&self, _goal: &GoalComponent, agent_world_context: &mut AgentGoalWorldContext) {
        agent_world_context.current_world_state[WorldStateProperty::IsRecoveringFromAttack] =
            Some(WSProperty::Truth(true));
    }
}
