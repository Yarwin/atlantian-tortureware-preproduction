use std::sync::{Arc, Mutex};
use godot::obj::InstanceId;
use crate::actions::action_types::Action;
use crate::ai::blackboard::Blackboard;
use crate::ai::working_memory::WorkingMemory;
use crate::ai::world_state::{WorldState};
use crate::goals::goal_component::GoalComponent;
use crate::godot_api::godot_thinker::GodotThinker;
use crate::sensors::sensor_types::{EventSensor, PollingSensor};
use crate::thinker_states::types::ThinkerState;
use godot::prelude::*;
use crate::animations::animation_data::AnimationsData;
use crate::targeting::targeting_systems::TargetMask;
use crate::thinker_states::navigation_subsystem::Navigator;

#[derive(Default, Debug)]
pub struct Thinker {
    pub id: u32,
    pub base_id: Option<InstanceId>,
    pub base: Option<Gd<GodotThinker>>,
    pub is_active: bool,
    pub state: Option<Box<dyn ThinkerState>>,

    /// shared mutable
    // todo – move to one struct and gate behind one Arc<Mutex<T>>
    pub shared: Arc<Mutex<ThinkerShared>>,

    /// shared immutable
    pub goals: Arc<Vec<GoalComponent>>,
    pub actions: Arc<Vec<Action>>,
    pub animations: Arc<AnimationsData>,

    pub polling_sensors: Vec<PollingSensor>,
    pub event_sensor: Vec<EventSensor>,
    pub navigation_map_rid: Option<Rid>,
    pub navigation_data: Navigator
}

/// a struct that keeps Thinker's components that are supposed to be shared between threads.
#[derive(Debug, Default)]
pub struct ThinkerShared {
    pub working_memory: WorkingMemory,
    pub blackboard: Blackboard,
    pub world_state: WorldState,
    pub target_mask: TargetMask
}
