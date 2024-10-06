use godot::prelude::godot_print;
use crate::ai::working_memory::{FactQuery, FactQueryCheck, WMDesireType};
use serde::{Deserialize, Serialize};
use crate::animations::animation_data::AnimationType;
use crate::goap_goals::goal_component::GoalComponent;
use crate::goap_goals::goal_types::{AgentGoalWorldContext, GoalBehaviour};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SatisfyDesireByPlayingAnimationGoal {
    pub desire_type: WMDesireType,
    pub animation_type: AnimationType
}

impl GoalBehaviour for SatisfyDesireByPlayingAnimationGoal {
    fn is_valid(&self, _goal: &GoalComponent, agent_world_context: &AgentGoalWorldContext) -> bool {
        let fact_query = FactQuery::with_check(FactQueryCheck::Desire(self.desire_type));
        if agent_world_context.working_memory.find_fact(fact_query).is_some() {
            return true
        }
        false
    }

    // set what animation to play to satisfy surprised goal
    fn activate(&self, _goal: &GoalComponent, agent_world_context: &mut AgentGoalWorldContext) -> bool {
        agent_world_context.blackboard.animation_target = Some(self.animation_type);
        true
    }

    fn deactivate(&self, _goal: &GoalComponent, agent_world_context: &mut AgentGoalWorldContext) {
        agent_world_context.blackboard.animation_target = None;
        let fact_query = FactQuery::with_check(FactQueryCheck::Desire(self.desire_type));
        agent_world_context.working_memory.mark_as_invalid(fact_query);
    }
}
