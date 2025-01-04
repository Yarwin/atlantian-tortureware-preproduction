#![allow(warnings, unused)]
use crate::ai::blackboard::SpeedMod;
use crate::ai::world_state::WSProperty::Truth;
use crate::ai::world_state::WorldState;
use crate::ai::world_state::WorldStateProperty::IsWeaponArmed;
use crate::goap_actions::action_component::ActionComponent;
use crate::goap_actions::action_types::{AgentActionPlanContext, AgentActionWorldContext};
use crate::targeting::target::AITarget;
use crate::thinker_states::animate::AnimateState;
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
    action_arguments.blackboard.animation_completed = false;
    let animation = action_arguments
        .blackboard
        .animation_target
        .as_ref()
        .unwrap_or(&inner.animation);
    let animation_props = &action_arguments.animations[animation];
    let new_state = AnimateState::new_boxed(
        animation_props.tree_name.clone(),
        animation_props.name.clone(),
        animation_props.mode.clone(),
    );
    action_arguments.blackboard.new_state = Some(new_state);
    let Some(AITarget::Character(_i, pos)) = action_arguments.blackboard.target.as_ref() else {
        return;
    };
}

pub fn finish(inner: &ActionComponent, action_arguments: AgentActionWorldContext) {
    if action_arguments.blackboard.animation_completed {
        action_arguments.blackboard.animation_completed = false;
        action_arguments.current_world_state[IsWeaponArmed] = Some(Truth(true));
    }
}

pub fn is_action_complete(
    inner: &ActionComponent,
    action_arguments: &AgentActionWorldContext,
) -> bool {
    return action_arguments.blackboard.animation_completed;
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
