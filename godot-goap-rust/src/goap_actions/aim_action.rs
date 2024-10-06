use godot::prelude::*;
use godot::classes::{Node3D};
use serde::{Deserialize, Serialize};
use crate::goap_actions::action_component::ActionComponent;
use crate::goap_actions::action_types::{ActionBehavior, AgentActionPlanContext, AgentActionWorldContext};
use crate::ai::blackboard::SpeedMod;
use crate::ai::world_state::WorldState;
use crate::targeting::target::AITarget;
use crate::thinker_states::animate::AnimateState;
use crate::thinker_states::navigation_subsystem::RotationTarget;


#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct AimWeapon;


impl ActionBehavior for AimWeapon {
    fn execute_action(&self, inner: &ActionComponent, action_arguments: AgentActionWorldContext) {
        action_arguments.blackboard.animation_completed = false;
        let Some(AITarget::Character(i, ..)) = action_arguments.blackboard.target.as_ref() else {return;};
        action_arguments.blackboard.rotation_target = Some(RotationTarget::Character(*i));
        action_arguments.blackboard.rotation_speed = SpeedMod::Fast;
        let animation = action_arguments.blackboard.animation_target.as_ref().unwrap_or(&inner.animation);
        let animation_props = &action_arguments.animations[animation];
        let new_state = AnimateState::new_boxed(animation_props.tree_name.clone(), animation_props.name.clone(), animation_props.mode.clone());
        action_arguments.blackboard.new_state = Some(new_state);
    }

    fn finish(&self, action_arguments: AgentActionWorldContext) {
        action_arguments.blackboard.invalidate_target = true;
        action_arguments.blackboard.animation_completed = false;
    }

    fn is_action_complete(&self, action_arguments: &AgentActionWorldContext) -> bool {
        action_arguments.blackboard.animation_completed
    }
}
