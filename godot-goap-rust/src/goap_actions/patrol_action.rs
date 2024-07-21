#![allow(warnings, unused)]

use crate::goap_actions::action_component::ActionComponent;
use crate::goap_actions::action_types::{AgentActionPlanContext, AgentActionWorldContext};
use crate::ai::world_state::WorldState;
use crate::ai_nodes::ai_node::AINode;
use crate::animations::animation_data::{AnimationProps, AnimationType};
use crate::targeting::target::AITarget;
use crate::thinker_states::animate::AnimateState;
use godot::prelude::*;
use crate::ai::blackboard::NavigationTarget;
use crate::thinker_states::navigation_subsystem::RotationTarget;

pub fn get_effects<'a>(
    inner: &'a ActionComponent,
    _action_arguments: &'a AgentActionPlanContext,
) -> &'a WorldState {
    &inner.effects
}

pub fn get_preconditions(inner: &ActionComponent) -> &WorldState {
    &inner.preconditions
}

pub fn get_cost(inner: &ActionComponent, _action_arguments: &AgentActionPlanContext) -> u32 {
    inner.cost
}

pub fn execute_action(inner: &ActionComponent, action_arguments: AgentActionWorldContext) {
    let rotation_target: Option<Vector3>;
    let Some(NavigationTarget::PatrolPoint(ainode_id, _p)) =
        *&action_arguments.blackboard.navigation_target.as_ref()
    else {
        return;
    };
    let Ok(mut ainodes_guard) = action_arguments.ai_nodes.as_mut().unwrap().read() else {panic!("mutex failed!")};
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
        action_arguments.blackboard.rotation_target = Some(RotationTarget::Position(*rotation_target));
    }
    action_arguments.blackboard.animation_completed = false;
    let patrol_animation_props = &action_arguments.animations[AnimationType::Patrol];
    let new_state = AnimateState::new_boxed(patrol_animation_props.tree_name.clone(), patrol_animation_props.name.clone(), patrol_animation_props.mode.clone());
    action_arguments.blackboard.new_state = Some(new_state);
}

pub fn finish(inner: &ActionComponent, action_arguments: AgentActionWorldContext) {
    action_arguments.blackboard.rotation_target = None;
    action_arguments.blackboard.animation_completed = false;
}

pub fn is_action_complete(
    inner: &ActionComponent,
    action_arguments: &AgentActionWorldContext,
) -> bool {
    if action_arguments.blackboard.animation_completed {
        return true;
    }

    false
}

pub fn is_action_interruptible(
    inner: &ActionComponent,
    action_arguments: &AgentActionWorldContext,
) -> bool {
    true
}

pub fn check_procedural_preconditions(
    inner: &ActionComponent,
    action_arguments: &AgentActionPlanContext,
) -> bool {
    true
}
