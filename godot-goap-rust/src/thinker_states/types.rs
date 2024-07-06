use crate::ai::blackboard::Blackboard;
use crate::ai::working_memory::WorkingMemory;
use crate::ai::world_state::WorldState;
use crate::godot_api::godot_thinker::GodotThinker;
use godot::prelude::*;
use std::fmt::Debug;

pub struct StateArguments<'a> {
    pub base: Gd<GodotThinker>,
    pub delta: f64,
    pub world_state: &'a mut WorldState,
    pub working_memory: &'a mut WorkingMemory,
    pub blackboard: &'a mut Blackboard,
}

pub trait ThinkerState: Debug {
    fn exit(&mut self, _args: &mut StateArguments) {}
    fn enter(&mut self, _args: StateArguments) {}
    fn physics_process(&mut self, delta: f64, args: StateArguments);
    fn update_animation(&mut self, args: StateArguments);
}
