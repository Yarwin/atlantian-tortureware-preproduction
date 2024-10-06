use serde::{Deserialize, Serialize};
use crate::goap_actions::action_component::ActionComponent;
use crate::goap_actions::action_types::{ActionBehavior, AgentActionWorldContext};
use crate::ai::blackboard::SpeedMod;
use crate::goap_actions::utils::action_set_animate_state;


#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct RangedAttack;

impl ActionBehavior for RangedAttack {
    fn execute_action(&self, inner: &ActionComponent, mut action_arguments: AgentActionWorldContext) {
        action_set_animate_state(inner, &mut action_arguments);
    }

    fn finish(&self, action_arguments: AgentActionWorldContext) {
        action_arguments.blackboard.rotation_speed = SpeedMod::Normal;
        action_arguments.blackboard.invalidate_target = true;
        action_arguments.blackboard.rotation_target = None;
        action_arguments.blackboard.animation_completed = false;
    }

    fn is_action_complete(&self, action_arguments: &AgentActionWorldContext) -> bool {
        action_arguments.blackboard.animation_completed
    }
}
