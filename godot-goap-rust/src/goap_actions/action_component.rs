use crate::ai::world_state::WorldState;
use crate::animations::animation_data::AnimationType;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use crate::ai::planner::PlanAction;
use crate::goap_actions::action_types::{Action, ActionType, AgentActionPlanContext};
use crate::goap_actions::action_types::ActionBehavior;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActionComponent {
    pub name: String,
    pub cost: u32,
    pub preconditions: WorldState,
    pub effects: WorldState,
    pub animation: AnimationType,
    pub action_type: Action
}

impl Hash for ActionComponent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.preconditions.hash(state);
        self.effects.hash(state);
    }
}

impl Eq for ActionComponent {}

impl PartialEq for ActionComponent {
    fn eq(&self, other: &Self) -> bool {
        let are_actions_the_same = ActionType::from(&self.action_type) == ActionType::from(&other.action_type);
        if !are_actions_the_same {
            return false
        }

        let are_preconditions_the_same = self
            .preconditions
            .count_state_differences(&other.preconditions)
            == 0
            && self.effects.count_state_differences(&other.effects) == 0;

        if !are_preconditions_the_same {
            return false
        }

        true
    }
}

impl PlanAction<AgentActionPlanContext<'_>> for ActionComponent {
    fn get_action_preconditions(&self) -> &WorldState {
        &self.preconditions
    }

    fn check_action_procedural_preconditions(&self, action_arguments: &AgentActionPlanContext<'_>) -> bool {
        self.action_type.check_procedural_preconditions(action_arguments)
    }

    #[allow(unused_variables)]
    fn get_action_effects<'a, 'b: 'a>(&'a self, action_arguments: &'b AgentActionPlanContext<'_>) -> &'a WorldState {
        &self.effects
    }

    fn get_action_cost(&self, action_arguments: &AgentActionPlanContext<'_>) -> u32 {
        self.cost + self.action_type.get_cost(action_arguments)
    }
}
