use godot::prelude::*;
use godot::classes::{CharacterBody3D, CollisionShape3D, PhysicsBody3D};
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
    /// default gravity multiplier
    #[export]
    gravity_scale: f32,
    #[export]
    jump_force: f32,
    /// a direction with a desired speed multiplier
    #[var]
    direction: Vector3,
    pub(crate) movement_data: Option<MovementData>,
    base: Base<CharacterBody3D>
}


impl CharacterController3D {
    pub fn get_motion_params(&self) -> MovementParameters {
        let jump_force = if let Some(previous_movement) = self.movement_data.as_ref() {
            if previous_movement.grounded && !self.direction.y.is_zero_approx() {
                self.jump_force * self.direction.y
            } else {
                0.0
            }
        } else {
            0.0
        };
        MovementParameters {
            direction: self.direction * Vector3::new(1.0, 0.0, 1.0),
            jump_force,
            body: self.base().clone().upcast::<PhysicsBody3D>(),
            collision_shape: self.collision_shape.as_ref().unwrap().clone(),
            // check if caching an array and cloning it wouldn't be better choice than creating new array every single time
            excluded_bodies: Some(array![self.base().get_rid()]),
            deceleration: self.deceleration,
            speed: self.speed,
            acceleration: self.acceleration,
            gravity_scale: self.gravity_scale,
        }
    }
}



#[godot_api]
impl CharacterController3D {
    #[func]
    pub fn process_movement(&mut self, delta: f64) {
        let motion_params = self.get_motion_params();
        self.movement_data = process_movement(delta as f32, motion_params, self.movement_data.take());
        let velocity = self.movement_data.as_ref().map(|md| md.velocity).unwrap_or(Vector3::ZERO);
        self.base_mut().set_velocity(velocity);
    }
}
