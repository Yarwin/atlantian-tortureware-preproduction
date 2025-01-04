use crate::ai::blackboard::{NavigationTarget, SpeedMod};
use crate::goap_goals::goal_component::GoalComponent;
use crate::goap_goals::goal_types::{AgentGoalWorldContext, GoalBehaviour};
use crate::targeting::target::AITarget;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ChaseEnemyGoal;

impl GoalBehaviour for ChaseEnemyGoal {
    fn is_valid(&self, _goal: &GoalComponent, agent_world_context: &AgentGoalWorldContext) -> bool {
        let Some(AITarget::Character(..)) = agent_world_context.blackboard.target else {
            return false;
        };
        true
    }

    fn activate(
        &self,
        _goal: &GoalComponent,
        agent_world_context: &mut AgentGoalWorldContext,
    ) -> bool {
        let Some(AITarget::Character(i, ..)) = agent_world_context.blackboard.target.as_ref()
        else {
            return false;
        };
        agent_world_context.blackboard.walk_speed = SpeedMod::Normal;
        agent_world_context.blackboard.navigation_target = Some(NavigationTarget::Character(*i));
        true
    }

    fn deactivate(&self, _goal: &GoalComponent, agent_world_context: &mut AgentGoalWorldContext) {
        agent_world_context.blackboard.navigation_target = None;
    }
}
