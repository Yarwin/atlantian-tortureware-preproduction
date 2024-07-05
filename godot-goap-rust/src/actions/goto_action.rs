#![allow(warnings, unused)]

use crate::actions::action_types::{AgentActionPlanContext, AgentActionWorldContext};
use crate::actions::action_component::ActionComponent;
use crate::ai::world_state::{WorldState, WorldStateProperty, WSProperty};
use godot::prelude::*;
use godot::classes::{NavigationServer3D};
use crate::targeting::target::Target;
use crate::thinker_states::goto::{Destination, GotoState};


fn get_destination(target: &Target) -> Destination {
    match target {
        Target::Character(_) => {
            unimplemented!()
        }
        Target::CombatOpportunity => {unimplemented!()}
        Target::Disturbance(_) => {unimplemented!()}
        Target::Interest(_) => {unimplemented!()}
        Target::Object(_) => {unimplemented!()}
        Target::SmartObject(_) => {unimplemented!()}
        Target::PatrolPoint(_node, pos) => {
            Destination::Position(pos.clone())
        }
    }
}

pub fn get_effects<'a>(inner: &'a ActionComponent, _action_arguments: &'a AgentActionPlanContext) -> &'a WorldState {
    &inner.effects
}

pub fn get_preconditions(inner: &ActionComponent) -> &WorldState {
    &inner.preconditions
}

pub fn get_cost(inner: &ActionComponent, _action_arguments: &AgentActionPlanContext) -> u32 {
    inner.cost
}

pub fn execute_action(inner: &ActionComponent, mut action_arguments: AgentActionWorldContext) {
    action_arguments.current_world_state[WorldStateProperty::IsNavigationFinished] = Some(WSProperty::Truth(false));
    let Some(target) = action_arguments.blackboard.target.as_ref().map(|t| get_destination(&t)) else {panic!("no target")};
    let new_state = Box::new(GotoState {
        destination: target,
        is_destination_blocked: false,
        finished: false
    });
    action_arguments.blackboard.new_state = Some(new_state);
}

pub fn finish(inner: &ActionComponent, action_arguments: AgentActionWorldContext) {
    action_arguments.current_world_state[WorldStateProperty::IsNavigationFinished] = None;
    action_arguments.blackboard.rotation_target = None;
}


pub fn is_action_complete(inner: &ActionComponent, action_arguments: &AgentActionWorldContext) -> bool {
    if let Some(WSProperty::Truth(is_finished)) = action_arguments.current_world_state[WorldStateProperty::IsNavigationFinished] {
        if is_finished {
            return true
        }
    }
    false
}


pub fn is_action_interruptible(inner: &ActionComponent, action_arguments: &AgentActionWorldContext) -> bool {
    true
}

pub fn check_procedural_preconditions(inner: &ActionComponent, action_arguments: &AgentActionPlanContext) -> bool {
    let Some(map_rid) = action_arguments.navigation_map_rid else {return false};
    let Some(target) = action_arguments.blackboard.target.as_ref().map(|t| get_destination(&t)) else {return false};
    let start_pos = action_arguments.blackboard.thinker_position;
    if let Destination::Position(target_pos) = target {
        let navpath: PackedVector3Array = NavigationServer3D::singleton().map_get_path(map_rid, start_pos, target_pos, false);
        if navpath.is_empty() {
            return false
        }
        return true
    }
    false
}
