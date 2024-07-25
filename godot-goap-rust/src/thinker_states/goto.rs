use std::time::SystemTime;
use crate::ai::blackboard::SpeedMod;
use crate::ai::world_state::{WSProperty, WorldStateProperty};
use crate::thinker_states::types::{StateArguments, ThinkerState};
use godot::classes::{AnimationNodeStateMachinePlayback, MeshInstance3D};
use godot::prelude::*;
use crate::thinker_states::navigation_subsystem::RotationTarget;

#[derive(Debug)]
pub enum Destination {
    Position(Vector3),
    Node(InstanceId),
    Character(InstanceId),
}

impl Destination {
    pub fn is_dynamic_pos(&self) -> bool {
        match self {
            Destination::Position(..) | Destination::Node(..) => false,
            Destination::Character(..) => true
        }
    }
}

#[derive(Debug)]
pub struct GotoState {
    pub destination: Destination,
    pub animation_name: String,
    pub is_destination_blocked: bool,
    pub finished: bool,
    pub should_repath: bool,
    pub time_since_last_pathing: SystemTime
}

impl GotoState {
    pub fn new_boxed(animation_name: String, destination: Destination) -> Box<Self> {
        let state = GotoState {
            destination,
            animation_name,
            is_destination_blocked: false,
            finished: false,
            should_repath: false,
            time_since_last_pathing: SystemTime::now()
        };
        Box::new(state)
    }
    fn get_target_pos(&self) -> Vector3 {
        match self.destination {
            Destination::Position(pos) => pos,
            Destination::Node(_id) => {
                todo!()
            }
            Destination::Character(id) => {
                let char: Gd<Node3D> = Gd::from_instance_id(id);
                char.get_global_position()
            }
        }
    }
}

impl ThinkerState for GotoState {
    fn exit(&mut self, args: &mut StateArguments) {
        let mut bind = args.base.bind_mut();
        if let Some(character) = bind.character_body.as_mut() {
            character.set_velocity(Vector3::ZERO);
        }
    }

    fn enter(&mut self, mut args: StateArguments) {
        self.should_repath = self.destination.is_dynamic_pos();
        let mut bind = args.base.bind_mut();
        let Some(nav_agent) = bind.navigation_agent.as_mut() else {
            return;
        };
        nav_agent.set_target_position(self.get_target_pos());
        let Some(anim_tree) = bind.animation_tree.as_mut() else {
            return;
        };
        let mut anim_node_state_machine = anim_tree
            .get("parameters/playback".into())
            .to::<Gd<AnimationNodeStateMachinePlayback>>();
        anim_node_state_machine.travel(self.animation_name.clone().into());
    }

    fn physics_process(&mut self, _delta: f64, mut args: StateArguments) {
        let mut bind = args.base.bind_mut();
        let Some(character) = bind.character_body.clone() else {
            return;
        };
        let Some(mut nav_agent) = bind.navigation_agent.as_mut().cloned() else {
            return;
        };

        if self.should_repath && (self.time_since_last_pathing.elapsed().unwrap().as_secs_f64() > 0.25) {
            self.time_since_last_pathing = SystemTime::now();
            nav_agent.set_target_position(self.get_target_pos());
        }

        let velocity = character.get_velocity();
        let speed = match args.blackboard.walk_speed {
            SpeedMod::Slow => bind.movement_speed_multiplier * bind.walk_speed_mod,
            SpeedMod::Normal => bind.movement_speed_multiplier,
            SpeedMod::Fast => bind.movement_speed_multiplier * bind.dash_speed_mod,
        };

        // bail if navigation finished
        if self.finished {
            // slow down
            if velocity.length() > 0.4 {
                args.blackboard.desired_velocity = Some(Vector3::ZERO);
                args.blackboard.thinker_position = character.get_global_position();
            }
            return;
        }
        if nav_agent.is_navigation_finished() {
            self.finished = true;
            args.world_state[WorldStateProperty::IsNavigationFinished] =
                Some(WSProperty::Truth(true));
            return;
        }

        let ground_offset = (character.get_global_transform().origin * Vector3::UP).y;
        let lateral_plane = Plane::new(Vector3::UP, ground_offset);
        let next_path_position: Vector3 = nav_agent.get_next_path_position();
        let direction: Vector3 = character
            .get_global_position()
            .direction_to(lateral_plane.project(next_path_position));
        if direction.length_squared().is_zero_approx() {
            return;
        }
        let look_target = lateral_plane.project(next_path_position);
        let mut debug_node = character.get_node_as::<MeshInstance3D>("Debug/DebugNav");
        debug_node.set_global_position(look_target);

        if let Some(RotationTarget::Position(current_look_target)) = args.blackboard.rotation_target.as_ref() {
            if !(*current_look_target - look_target).is_zero_approx() {
                args.blackboard.rotation_target = Some(RotationTarget::Position(look_target));
            }
        } else {
            args.blackboard.rotation_target = Some(RotationTarget::Position(look_target));
        }

        args.blackboard.desired_velocity = Some(direction * speed);
        args.blackboard.thinker_position = character.get_global_position();
    }

    fn update_animation(&mut self, _args: StateArguments) {
        todo!()
    }
}
