use crate::goap_goals::goal_types::{AgentGoalWorldContext, GoalBehaviour};
use serde::{Deserialize, Serialize};
use crate::ai::working_memory::Event::AttackPerformed;
use crate::ai::working_memory::WMProperty;
use crate::ai::world_state::WorldStateProperty;
use crate::ai::world_state::WSProperty::Truth;
use crate::goap_goals::goal_component::GoalComponent;


#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ExecuteAttackGoal;

impl GoalBehaviour for ExecuteAttackGoal {
    fn is_valid(&self, _goal: &GoalComponent, agent_world_context: &AgentGoalWorldContext) -> bool {
        agent_world_context.blackboard.chosen_attack_idx.is_some()
    }

    fn deactivate(&self, _goal: &GoalComponent, agent_world_context: &mut AgentGoalWorldContext) {
        let Some(attack_idx) = agent_world_context.blackboard.chosen_attack_idx.take() else {return;};
        let property = WMProperty::Event(AttackPerformed {id: attack_idx});
        agent_world_context.working_memory.add_or_update(property, 1.0, 30.0);
        agent_world_context.current_world_state[WorldStateProperty::HasAttack] = Some(Truth(false));
        agent_world_context.blackboard.invalidate_attack = true;
    }
}
