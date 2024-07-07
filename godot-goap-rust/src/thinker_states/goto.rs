use crate::ai::blackboard::MovementSpeed;
use crate::ai::world_state::{WSProperty, WorldStateProperty};
use crate::thinker_states::types::{StateArguments, ThinkerState};
use godot::classes::AnimationNodeStateMachinePlayback;
use godot::prelude::*;

#[derive(Debug)]
pub enum Destination {
    Position(Vector3),
    Node(InstanceId),
    Character(InstanceId),
}

#[derive(Debug)]
pub struct GotoState {
    pub destination: Destination,
    pub is_destination_blocked: bool,
    pub finished: bool,
}

impl GotoState {
    fn get_target_pos(&self) -> Vector3 {
        match self.destination {
            Destination::Position(pos) => pos,
            Destination::Node(_id) => {
                todo!()
            }
            Destination::Character(_id) => {
                todo!()
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
        anim_node_state_machine.travel("Walk".into());
    }

    fn physics_process(&mut self, delta: f64, mut args: StateArguments) {
        let mut bind = args.base.bind_mut();
        let Some(character) = bind.character_body.clone() else {
            return;
        };
        let Some(mut anim_tree) = bind.animation_tree.clone() else {
            return;
        };
        let velocity = character.get_velocity();
        let (speed, acceleration) = match args.blackboard.walk_speed {
            MovementSpeed::Invalid => {
                godot_print!("invalid!!");
                (bind.movement_speed, bind.acceleration)
            }
            MovementSpeed::Walk => (
                bind.movement_speed * bind.walk_speed_mod,
                bind.acceleration * bind.walk_speed_mod,
            ),
            MovementSpeed::Run => (bind.movement_speed, bind.acceleration),
            MovementSpeed::Dash => (bind.movement_speed * 2.0, bind.acceleration * 2.0),
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
        let Some(nav_agent) = bind.navigation_agent.as_mut() else {
            return;
        };
        if nav_agent.is_navigation_finished() {
            anim_tree.set("walk".into(), false.to_variant());
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
        let update_look_target = args
            .blackboard
            .rotation_target
            .as_ref()
            .map(|rt| !(*rt - look_target).is_zero_approx())
            .unwrap_or(true);
        if update_look_target {
            args.blackboard.rotation_target = Some(look_target);
        }

        let dot_product: f32 = 1.0;
        // let mut dot_product: f32 = character.get_transform().basis.col_c().dot(direction).max(0.0);
        // if dot_product > 0.9 {
        //     dot_product = 1.0;
        // }
        let movspeed =
            velocity.move_toward(direction * speed * dot_product, acceleration * delta as f32);
        args.blackboard.desired_velocity = Some(movspeed);
        args.blackboard.thinker_position = character.get_global_position();
    }

    fn update_animation(&mut self, _args: StateArguments) {
        todo!()
    }
}
