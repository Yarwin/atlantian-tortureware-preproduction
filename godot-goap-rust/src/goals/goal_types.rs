use crate::ai::blackboard::Blackboard;
use crate::ai::working_memory::WorkingMemory;
use crate::ai::world_state::WorldState;
use crate::ai_nodes::ai_node::AINode;
use crate::goals::basic_goal::BasicGoal;
use crate::goals::dodge_goal::DodgeGoal;
use crate::goals::goal_component::GoalComponent;
use crate::goals::kill_enemy_goal::KillEnemyGoal;
use crate::goals::patrol_goal::PatrolGoal;
use crate::goals::surprised_by_enemy_goal::BeSurprisedByEnemyGoal;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct AgentGoalWorldContext<'a> {
    pub id: &'a u32,
    pub working_memory: &'a mut WorkingMemory,
    pub current_world_state: &'a mut WorldState,
    pub blackboard: &'a mut Blackboard,
    pub ai_nodes: &'a mut Option<Arc<Mutex<HashMap<u32, AINode>>>>,
}

#[allow(clippy::derivable_impls, clippy::enum_variant_names)]
#[enum_dispatch]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GoalType {
    /// a goal with constant/static relevance that only checks against given WorldState
    BasicGoal,
    DodgeGoal,
    KillEnemyGoal,
    PatrolGoal,
    BeSurprisedByEnemyGoal,
}

#[enum_dispatch(GoalType)]
pub trait GoalBehaviour {
    fn validate_context(
        &self,
        goal: &GoalComponent,
        agent_world_context: &AgentGoalWorldContext,
    ) -> bool {
        let required_state_meet = goal
            .required_state
            .count_unsatisfied_world_state_props(agent_world_context.current_world_state)
            == 0;
        let desired_state_not_meet = goal
            .desired_state
            .count_unsatisfied_world_state_props(agent_world_context.current_world_state)
            != 0;
        required_state_meet && desired_state_not_meet
    }

    fn is_valid(
        &self,
        _goal: &GoalComponent,
        _agent_world_context: &AgentGoalWorldContext,
    ) -> bool {
        true
    }

    fn calculate_goal_relevance(
        &self,
        goal: &GoalComponent,
        _agent_world_context: &AgentGoalWorldContext,
    ) -> u32 {
        goal.priority
    }

    fn activate(
        &self,
        _goal: &GoalComponent,
        _agent_world_context: &mut AgentGoalWorldContext,
    ) -> bool {
        true
    }

    fn deactivate(&self, _goal: &GoalComponent, _agent_world_context: &mut AgentGoalWorldContext) {}
}
