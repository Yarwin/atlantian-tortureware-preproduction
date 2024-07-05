#![allow(warnings, unused)]

use godot::prelude::*;
use crate::actions::action_types::{AgentActionPlanContext, AgentActionWorldContext};
use crate::actions::action_component::ActionComponent;
use crate::ai::world_state::WorldState;
use crate::ai_nodes::ai_node::AINode;
use crate::animations::animation_data::{AnimationProps, AnimationType};
use crate::targeting::target::Target;
use crate::thinker_states::animate::AnimateState;


pub fn get_effects<'a>(inner: &'a ActionComponent, _action_arguments: &'a AgentActionPlanContext) -> &'a WorldState {
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
    let Some(Target::PatrolPoint(object_handle, _p)) = *&action_arguments.blackboard.target.as_ref() else {return;};
    let Ok(ainode) = object_handle.lock() else {panic!("mutex poisoned!")};
    let AINode::Patrol {base: _, next: _, orientation} = &*ainode else { return; };
    if let Some(rotation_target) = orientation {}
    rotation_target = orientation.clone();
    drop(ainode);
    action_arguments.blackboard.rotation_target = rotation_target;
    action_arguments.blackboard.animation_completed = false;
    let patrol_animation_props = &action_arguments.animations[AnimationType::Patrol];
    let new_state = Box::new(
        AnimateState {
            name: patrol_animation_props.name.clone(),
            loops: patrol_animation_props.loops,
            cyclic: patrol_animation_props.cyclic,
            total_time: patrol_animation_props.loop_time,
            elapsed_time: 0.0,
        });
    action_arguments.blackboard.new_state = Some(new_state);
}


pub fn finish(inner: &ActionComponent, action_arguments: AgentActionWorldContext) {
    action_arguments.blackboard.rotation_target = None;
    action_arguments.blackboard.animation_completed = false;
}

pub fn is_action_complete(inner: &ActionComponent, action_arguments: &AgentActionWorldContext) -> bool {

    if action_arguments.blackboard.animation_completed {
        return true
    }

    false
}

pub fn is_action_interruptible(inner: &ActionComponent, action_arguments: &AgentActionWorldContext) -> bool {
    true
}

pub fn check_procedural_preconditions(inner: &ActionComponent, action_arguments: &AgentActionPlanContext) -> bool {
    true
}

