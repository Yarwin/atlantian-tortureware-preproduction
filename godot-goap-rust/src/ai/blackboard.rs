// BlackBoard is used by AI subsystems to share their requests, intents, and results.

use crate::targeting::target::Target;
use crate::thinker_states::types::ThinkerState;
use godot::prelude::*;
use std::collections::VecDeque;
use crate::animations::animation_data::AnimationType;
use crate::targeting::targeting_systems::TargetMask;

#[derive(Default, Debug)]
pub enum Awareness {
    #[default]
    Unaware,
    Alert,
}

#[derive(Default, Debug)]
pub enum MovementSpeed {
    #[default]
    Invalid,
    Walk,
    Run,
    Dash,
}

#[derive(Debug)]
pub enum NavigationTarget {
    /// patrol point
    PatrolPoint(u32, Vector3),
}

#[derive(Default, Debug)]
pub struct Blackboard {
    pub current_locked_node: Option<u32>,
    pub new_state: Option<Box<dyn ThinkerState + Send>>,
    pub current_plan_ids: VecDeque<usize>,
    pub current_goal: Option<usize>,
    pub thinker_position: Vector3,
    pub needs_retargeting: bool,
    pub target: Option<Target>,
    pub valid_targets: TargetMask,
    pub navigation_target: Option<NavigationTarget>,
    pub animation_target: Option<AnimationType>,
    pub invalidate_target: bool,
    pub rotation_target: Option<Vector3>,
    pub walk_speed: MovementSpeed,
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
}
