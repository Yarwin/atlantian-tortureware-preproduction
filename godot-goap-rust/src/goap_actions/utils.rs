use crate::goap_actions::action_component::ActionComponent;
use crate::goap_actions::action_types::AgentActionWorldContext;
use crate::thinker_states::animate::AnimateState;


pub fn action_set_animate_state(inner: &ActionComponent, action_arguments: &mut AgentActionWorldContext) {
    action_arguments.blackboard.animation_completed = false;
    let animation = action_arguments.blackboard.animation_target.as_ref().unwrap_or(&inner.animation);
    let animation_props = &action_arguments.animations[animation];
    let new_state = AnimateState::new_boxed(animation_props.tree_name.clone(), animation_props.name.clone(), animation_props.mode.clone());
    action_arguments.blackboard.new_state = Some(new_state);
}
