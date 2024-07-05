use godot::classes::{AnimationNodeStateMachinePlayback};
use godot::prelude::*;
use crate::thinker_states::types::{StateArguments, ThinkerState};

#[derive(Debug)]
#[allow(unused_attributes)]
pub struct AnimateState {
    pub name: String,
    pub loops: u32,
    pub cyclic: bool,
    pub total_time: f64,
    pub elapsed_time: f64,
}


impl ThinkerState for AnimateState {
    fn exit(&mut self, _args: &mut StateArguments) {
    }

    fn enter(&mut self, mut args: StateArguments) {
        let mut bind = args.base.bind_mut();
        let Some(anim_tree) = bind.animation_tree.as_mut() else { return; };
        let mut anim_node_state_machine = anim_tree.get("parameters/playback".into()).to::<Gd<AnimationNodeStateMachinePlayback>>();
        anim_node_state_machine.travel(self.name.clone().into());
    }

    fn physics_process(&mut self, delta: f64, args: StateArguments) {
        self.elapsed_time += delta;
        if self.elapsed_time > self.total_time {
            args.blackboard.animation_completed = true;
        }

    }

    fn update_animation(&mut self, _args: StateArguments) {
    }
}