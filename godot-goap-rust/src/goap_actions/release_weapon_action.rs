use crate::ai::world_state::WSProperty::Truth;
use crate::ai::world_state::WorldStateProperty::IsWeaponArmed;
use crate::goap_actions::action_component::ActionComponent;
use crate::goap_actions::action_types::{ActionBehavior, AgentActionWorldContext};
use crate::goap_actions::utils::action_set_animate_state;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct ReleaseWeapon;

impl ActionBehavior for ReleaseWeapon {
    fn execute_action(
        &self,
        inner: &ActionComponent,
        mut action_arguments: AgentActionWorldContext,
    ) {
        action_set_animate_state(inner, &mut action_arguments);
    }

    fn finish(&self, action_arguments: AgentActionWorldContext) {
        action_arguments.current_world_state[IsWeaponArmed] = Some(Truth(false));
        action_arguments.blackboard.animation_completed = false;
    }

    fn is_action_complete(&self, action_arguments: &AgentActionWorldContext) -> bool {
        action_arguments.blackboard.animation_completed
    }
}
