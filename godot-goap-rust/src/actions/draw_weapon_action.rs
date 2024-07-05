#![allow(warnings, unused)]
/// An action that prepares an enemy attack
use crate::actions::action_types::{AgentActionPlanContext, AgentActionWorldContext};
use crate::actions::action_component::ActionComponent;
use crate::ai::world_state::WorldState;


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
    todo!()
}

pub fn finish(inner: &ActionComponent, action_arguments: AgentActionWorldContext) {
    todo!()
}

pub fn is_action_complete(inner: &ActionComponent, action_arguments: &AgentActionWorldContext) -> bool {
    todo!()
}

pub fn is_action_interruptible(inner: &ActionComponent, action_arguments: &AgentActionWorldContext) -> bool {
    todo!()
}

pub fn check_procedural_preconditions(inner: &ActionComponent, action_arguments: &AgentActionPlanContext) -> bool {
    todo!()
}

