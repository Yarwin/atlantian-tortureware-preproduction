#![allow(warnings, unused)]

use crate::goap_actions::action_component::ActionComponent;
use crate::goap_actions::action_types::{AgentActionPlanContext, AgentActionWorldContext};
use crate::ai::world_state::{WSProperty, WorldState, WorldStateProperty};
use crate::targeting::target::AITarget;
use crate::thinker_states::goto::{Destination, GotoState};
use godot::classes::NavigationServer3D;
use godot::classes::CharacterBody3D;
use godot::prelude::*;
use crate::ai::blackboard::NavigationTarget;
use crate::animations::animation_data::AnimationType;

fn get_destination(target: &NavigationTarget) -> Destination {
    match target {
        NavigationTarget::PatrolPoint(ainode_id, pos) => {
            Destination::Position(*pos)}
        NavigationTarget::Character(instance_id) => {
            Destination::Character(*instance_id) }
    }
}

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

pub fn execute_action(inner: &ActionComponent, mut action_arguments: AgentActionWorldContext) {
    action_arguments.current_world_state[WorldStateProperty::IsNavigationFinished] =
        Some(WSProperty::Truth(false));
    let Some(target) = action_arguments
        .blackboard
        .navigation_target
        .as_ref()
        .map(|t| get_destination(&t))
    else {
        panic!("no target")
    };
    let anim = &action_arguments.animations[AnimationType::Walk];
    let new_state = GotoState::new_boxed(anim.tree_name.clone(), target);
    action_arguments.blackboard.new_state = Some(new_state);
}

pub fn finish(inner: &ActionComponent, action_arguments: AgentActionWorldContext) {
    action_arguments.current_world_state[WorldStateProperty::IsNavigationFinished] = None;
    action_arguments.blackboard.rotation_target = None;
}

pub fn is_action_complete(
    inner: &ActionComponent,
    action_arguments: &AgentActionWorldContext,
) -> bool {
    if let Some(WSProperty::Truth(is_finished)) =
        action_arguments.current_world_state[WorldStateProperty::IsNavigationFinished]
    {
        if is_finished {
            return true;
        }
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
    let Some(map_rid) = action_arguments.navigation_map_rid else {
        return false;
    };
    let Some(target) = action_arguments
        .blackboard
        .navigation_target
        .as_ref()
        .map(|t| get_destination(&t))
    else {
        return false;
    };
    let start_pos = action_arguments.blackboard.thinker_position;
    if let Destination::Position(target_pos) = target {
        let navpath: PackedVector3Array =
            NavigationServer3D::singleton().map_get_path(map_rid, start_pos, target_pos, false);
        if navpath.is_empty() {
            return false;
        }
        return true;
    }
    true
}
