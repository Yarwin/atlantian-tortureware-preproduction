#![allow(warnings, unused)]
use crate::goap_actions::action_component::ActionComponent;
use crate::goap_actions::action_types::{AgentActionPlanContext, AgentActionWorldContext};
use crate::ai::world_state::WorldState;
use crate::ai::world_state::WorldStateProperty::IsWeaponArmed;
use crate::ai::world_state::WSProperty::Truth;
use crate::thinker_states::animate::AnimateState;

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
    action_arguments.blackboard.animation_completed = false;
    // check for animation set by the goal or animate action's animation
    let animation = &inner.animation;
    let animation_props = &action_arguments.animations[animation];
    let new_state = AnimateState::new_boxed(animation_props.tree_name.clone(), animation_props.name.clone(), animation_props.mode.clone());
    action_arguments.blackboard.new_state = Some(new_state);
}

pub fn finish(inner: &ActionComponent, action_arguments: AgentActionWorldContext) {
    action_arguments.current_world_state[IsWeaponArmed] = Some(Truth(false));
    action_arguments.blackboard.animation_completed = false;
}

pub fn is_action_complete(
    inner: &ActionComponent,
    action_arguments: &AgentActionWorldContext,
) -> bool {
    return action_arguments.blackboard.animation_completed
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
