use serde::{Deserialize, Serialize};
use crate::ai::world_state::WorldStateProperty::IsWeaponArmed;
use crate::ai::world_state::WSProperty::Truth;
use crate::goap_actions::action_component::ActionComponent;
use crate::goap_actions::action_types::{ActionBehavior, AgentActionWorldContext};
use crate::goap_actions::utils::action_set_animate_state;

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct ArmWeapon;

impl ActionBehavior for ArmWeapon {
    fn execute_action(&self, inner: &ActionComponent, mut action_arguments: AgentActionWorldContext) {
        action_set_animate_state(inner, &mut action_arguments);
    }

    fn finish(&self, action_arguments: AgentActionWorldContext) {
        if action_arguments.blackboard.animation_completed {
            action_arguments.blackboard.animation_completed = false;
            action_arguments.current_world_state[IsWeaponArmed] = Some(Truth(true));
        }
    }

    fn is_action_complete(&self, action_arguments: &AgentActionWorldContext) -> bool {
        action_arguments.blackboard.animation_completed
    }
}
