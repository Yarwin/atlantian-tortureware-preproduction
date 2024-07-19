use godot::prelude::*;
use godot::classes::{CharacterBody3D, CollisionShape3D, ICharacterBody3D, PhysicsBody3D};
use crate::character_controler::movement_data::MovementData;
use crate::character_controler::process_movement::{MovementParameters, process_movement};


#[derive(GodotClass, Debug)]
#[class(init, base=CharacterBody3D)]
pub struct CharacterController3D {
    #[export]
    collision_shape: Option<Gd<CollisionShape3D>>,
    #[export]
    deceleration: f32,
    #[export]
    speed: f32,
    #[export]
    acceleration: f32,
    #[export]
    gravity_scale: f32,
    movement_data: Option<MovementData>,
    base: Base<CharacterBody3D>
}


impl CharacterController3D {
    pub fn get_motion_params(&self) -> MovementParameters {
        let input = Vector2::new(
            Input::singleton().get_action_strength("move_right".into()) - Input::singleton().get_action_strength("move_left".into()),
            Input::singleton().get_action_strength("move_forward".into()) - Input::singleton().get_action_strength("move_back".into()),
        ).normalized();
        let direction = input.x * self.base().get_basis().col_a() - input.y * self.base().get_basis().col_c();

        MovementParameters {
            direction,
            body: self.base().clone().upcast::<PhysicsBody3D>(),
            collision_shape: self.collision_shape.as_ref().unwrap().clone(),
            excluded_bodies: Some(array![self.base().get_rid()]),
            deceleration: self.deceleration,
            speed: self.speed,
            acceleration: self.acceleration,
            gravity_scale: self.gravity_scale,
        }
    }
}

#[godot_api]
impl ICharacterBody3D for CharacterController3D {
    fn physics_process(&mut self, delta: f64) {
        let motion_params = self.get_motion_params();
        self.movement_data = process_movement(delta as f32, motion_params, self.movement_data.take());
    }
}