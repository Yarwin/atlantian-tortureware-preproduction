#![allow(warnings, unused)]

use crate::ai::blackboard::NavigationTarget;
use crate::ai::world_state::WorldState;
use crate::ai_nodes::ai_node::AINode;
use crate::animations::animation_data::{AnimationProps, AnimationType};
use crate::goap_actions::action_component::ActionComponent;
use crate::goap_actions::action_types::{
    ActionBehavior, AgentActionPlanContext, AgentActionWorldContext,
};
use crate::targeting::target::AITarget;
use crate::thinker_states::animate::AnimateState;
use crate::thinker_states::navigation_subsystem::RotationTarget;
use godot::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct Patrol;

impl ActionBehavior for Patrol {
    fn execute_action(&self, inner: &ActionComponent, action_arguments: AgentActionWorldContext) {
        let rotation_target: Option<Vector3>;
        let Some(NavigationTarget::PatrolPoint(ainode_id, _p)) =
            *&action_arguments.blackboard.navigation_target.as_ref()
        else {
            return;
        };
        let Ok(mut ainodes_guard) = action_arguments.ai_nodes.as_mut().unwrap().read() else {
            panic!("couldn't find ainodes!")
        };
        let ainode = ainodes_guard.get(ainode_id).unwrap();
        let AINode::Patrol {
            base: _,
            next: _,
            orientation,
        } = &ainode
        else {
            return;
        };

        if let Some(rotation_target) = orientation {
            action_arguments.blackboard.rotation_target =
                Some(RotationTarget::Position(*rotation_target));
        }
        action_arguments.blackboard.animation_completed = false;
        let patrol_animation_props = &action_arguments.animations[AnimationType::Patrol];
        let new_state = AnimateState::new_boxed(
            patrol_animation_props.tree_name.clone(),
            patrol_animation_props.name.clone(),
            patrol_animation_props.mode.clone(),
        );
        action_arguments.blackboard.new_state = Some(new_state);
    }

    fn finish(&self, action_arguments: AgentActionWorldContext) {
        action_arguments.blackboard.rotation_target = None;
        action_arguments.blackboard.animation_completed = false;
    }

    fn is_action_complete(&self, action_arguments: &AgentActionWorldContext) -> bool {
        if action_arguments.blackboard.animation_completed {
            return true;
        }
        false
    }

    fn is_action_interruptible(&self, action_arguments: &AgentActionWorldContext) -> bool {
        true
    }

    fn check_procedural_preconditions(&self, action_arguments: &AgentActionPlanContext) -> bool {
        true
    }
}
