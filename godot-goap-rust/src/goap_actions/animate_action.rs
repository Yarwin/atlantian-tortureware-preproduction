#![allow(warnings, unused)]

use godot::prelude::godot_print;
use serde::{Deserialize, Serialize};
use crate::goap_actions::action_component::ActionComponent;
use crate::goap_actions::action_types::{ActionBehavior, AgentActionPlanContext, AgentActionWorldContext};
use crate::ai::working_memory::{Event, FactQuery, FactQueryCheck};
use crate::ai::world_state::WorldState;
use crate::animations::animation_data::AnimationType;
use crate::goap_actions::utils::action_set_animate_state;
use crate::thinker_states::animate::AnimateState;

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct Animate;

impl ActionBehavior for Animate {
    fn execute_action(&self, inner: &ActionComponent, mut action_arguments: AgentActionWorldContext) {
        action_set_animate_state(inner, &mut action_arguments);
    }

    fn finish(&self, action_arguments: AgentActionWorldContext) {
        action_arguments.blackboard.animation_completed = false;
    }

    fn is_action_complete(&self, action_arguments: &AgentActionWorldContext) -> bool {
        return action_arguments.blackboard.animation_completed;
    }
}
