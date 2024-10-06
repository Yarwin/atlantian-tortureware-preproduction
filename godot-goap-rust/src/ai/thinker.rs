use crate::ai::blackboard::Blackboard;
use crate::ai::working_memory::WorkingMemory;
use crate::ai::world_state::WorldState;
use crate::animations::animation_data::AnimationsData;
use crate::goap_goals::goal_component::GoalComponent;
use crate::godot_api::godot_thinker::GodotThinker;
use crate::sensors::sensor_types::{EventSensor, PollingSensor};
use crate::targeting::targeting_systems::TargetMask;
use crate::thinker_states::navigation_subsystem::Navigator;
use crate::thinker_states::types::ThinkerState;
use godot::obj::InstanceId;
use godot::prelude::*;
use std::sync::{Arc, Mutex};
use crate::goap_actions::action_component::ActionComponent;

#[derive(Default, Debug)]
pub struct Thinker {
    pub id: u32,
    pub base_id: Option<InstanceId>,
    pub base: Option<Gd<GodotThinker>>,
    pub is_active: bool,
    pub state: Option<Box<dyn ThinkerState>>,

    /// mutable data kept by the thinker shared with various subsystems (that might edit it)
    pub shared: Arc<Mutex<ThinkerShared>>,

    /// shared immutable owned by AIManager
    pub goals: Arc<Vec<GoalComponent>>,
    pub actions: Arc<Vec<ActionComponent>>,
    pub animations: Arc<AnimationsData>,
    pub polling_sensors: Vec<PollingSensor>,
    pub event_sensor: Vec<EventSensor>,
    pub navigation_map_rid: Option<Rid>,
    pub navigation_data: Navigator,
}

/// a struct that keeps Thinker's components that are supposed to be shared between threads.
#[derive(Debug, Default)]
pub struct ThinkerShared {
    pub working_memory: WorkingMemory,
    pub blackboard: Blackboard,
    pub world_state: WorldState,
    pub target_mask: TargetMask,
}
