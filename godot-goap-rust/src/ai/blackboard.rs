// BlackBoard is used by AI subsystems to share their requests, intents, and results.

use crate::animations::animation_data::AnimationType;
use crate::targeting::target::AITarget;
use crate::targeting::targeting_systems::TargetMask;
use crate::thinker_states::navigation_subsystem::RotationTarget;
use crate::thinker_states::types::ThinkerState;
use godot::prelude::*;
use std::collections::VecDeque;
use std::time::SystemTime;

#[derive(Default, Debug)]
pub enum Awareness {
    #[default]
    Unaware,
    Alert,
}

#[derive(Default, Debug)]
pub enum SpeedMod {
    #[default]
    Slow,
    Normal,
    Fast,
}

#[derive(Debug)]
pub enum NavigationTarget {
    /// patrol point
    PatrolPoint(u32, Vector3),
    Character(InstanceId),
}

#[derive(Debug)]
pub struct Failed {
    pub index: usize,
    pub time: SystemTime,
}

impl Failed {
    pub fn new(index: usize) -> Self {
        Failed {
            index,
            time: SystemTime::now(),
        }
    }
}

#[derive(Default, Debug)]
pub struct Blackboard {
    /// id of ainode locked by this entity
    pub current_locked_node: Option<u32>,
    /// pointer to some new state for thinker (Goto/animate).
    pub new_state: Option<Box<dyn ThinkerState + Send>>,
    pub current_plan_ids: VecDeque<usize>,
    pub current_goal: Option<usize>,
    /// todo – move it to Working Memory instead?
    pub failed_goals: Vec<Failed>,
    pub thinker_position: Vector3,
    pub target: Option<AITarget>,
    pub distance_to_target: Option<f32>,
    pub valid_targets: TargetMask,
    pub navigation_target: Option<NavigationTarget>,
    pub animation_target: Option<AnimationType>,
    pub invalidate_target: bool,
    pub invalidate_plan: bool,
    pub invalidate_attack: bool,
    pub chosen_attack_idx: Option<usize>,
    pub rotation_target: Option<RotationTarget>,
    pub walk_speed: SpeedMod,
    pub rotation_speed: SpeedMod,
    pub desired_velocity: Option<Vector3>,
    pub animation_completed: bool,
}

impl Blackboard {
    pub fn current_action(&self) -> Option<usize> {
        self.current_plan_ids.front().copied()
    }

    pub fn next_action(&mut self) -> Option<usize> {
        self.current_plan_ids.pop_front()
    }

    pub fn validate_failed(&mut self) {
        self.failed_goals
            .retain(|fail| fail.time.elapsed().unwrap().as_secs_f64() < 1.0);
    }

    pub fn is_goal_failed(&self, goal: usize) -> bool {
        self.failed_goals.iter().any(|failed| failed.index == goal)
    }
}
