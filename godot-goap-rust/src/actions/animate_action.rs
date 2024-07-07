#![allow(warnings, unused)]
use crate::actions::action_component::ActionComponent;
use crate::actions::action_types::{AgentActionPlanContext, AgentActionWorldContext};
use crate::ai::working_memory::{Event, FactQuery, FactQueryCheck};
use crate::ai::world_state::WorldState;
use crate::animations::animation_data::AnimationType;
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
    let animation = action_arguments.blackboard.animation_target.as_ref().expect("no animation set by a goal!");
    let animation_props = &action_arguments.animations[animation];
    let new_state = AnimateState::new_boxed(animation_props.name.clone(), animation_props.mode.clone());
    action_arguments.blackboard.new_state = Some(new_state);
}

pub fn finish(inner: &ActionComponent, action_arguments: AgentActionWorldContext) {
}

pub fn is_action_complete(
    inner: &ActionComponent,
    action_arguments: &AgentActionWorldContext,
) -> bool {
    return action_arguments.blackboard.animation_completed;
    // if action_arguments.blackboard.animation_completed {
    //
    // }
    // let fact_query = FactQuery::with_check(FactQueryCheck::Event(EventType::AnimationCompleted("a".into())));
    // if action_arguments.working_memory.find_fact(fact_query).is_some() {
    //     return true
    // }
    // return false
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
