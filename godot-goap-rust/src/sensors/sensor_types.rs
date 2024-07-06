use crate::ai::ai_stimulus::AIStimulusType;
use crate::ai::blackboard::Blackboard;
use crate::ai::working_memory::WorkingMemory;
use crate::ai::world_state::WorldState;
use crate::ai_nodes::ai_node::AINode;
use crate::sensors::damage_sensor::DamageSensor;
use crate::sensors::get_patrol_points_sensor::PatrolPointSensor;
use crate::sensors::vision_character_sensor::VisionCharacterSensor;
use crate::targeting::targeting_systems::TargetMask;
use crate::thinker_states::process_thinker::PollingResult;
use enum_dispatch::enum_dispatch;
use godot::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[allow(unused_attributes, dead_code)]
#[derive(Debug)]
pub struct SensorArguments<'a> {
    // pub base: Gd<GodotThinker>,
    pub character_rid: Rid,
    pub head_position: Vector3,
    pub thinker_forward_axis: Vector3,
    pub world_state: &'a mut WorldState,
    pub working_memory: &'a mut WorkingMemory,
    pub blackboard: &'a mut Blackboard,
    pub polls: &'a mut PollingResult,
    pub target_mask: &'a mut TargetMask,
    pub ainodes: &'a Arc<Mutex<HashMap<u32, AINode>>>,
}

#[enum_dispatch]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EventSensor {
    DamageSensor,
}

#[enum_dispatch]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PollingSensor {
    PatrolPointSensor,
    VisionCharacterSensor,
}

#[enum_dispatch(PollingSensor)]
pub trait SensorPolling {
    fn process(&mut self, delta: f64, args: &mut SensorArguments) -> bool;
}

#[enum_dispatch(EventSensor)]
pub trait SensorEvent {
    /// stimulate given sensor with given stim. Returns true if stimulus has been consumed
    fn stimulate(&mut self, _stim: AIStimulusType, _args: &mut SensorArguments) -> bool;
}
