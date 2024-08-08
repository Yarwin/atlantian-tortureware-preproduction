use std::f32::consts::FRAC_PI_2;
use std::f64::consts::PI;
use godot::classes::{InputEvent, InputEventMouseMotion, ShapeCast3D};
use godot::prelude::*;
use crate::character_controler::character_controller_3d::CharacterController3D;
use godot::classes::input::MouseMode;
use godot::global::{fmod, lerpf, sin};
use crate::act_react::act_react_executor::ActReactExecutor;
use crate::act_react::act_react_resource::ActReactResource;
use crate::act_react::react_area_3d::ActReactArea3D;
use crate::godot_api::gamesys::{GameSys, GameSystem};


#[derive(Debug, Default)]
pub struct CameraData {
    pub target_rotation_change_y: f32,
    pub target_rotation_head: Vector3,
    pub mouse_movement: Vector2,
    pub rotation_origins: Vector3,
    pub bobtimes: [f32; 3],
}


#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct PlayerCameraController3D {
    #[init(node = "../Head/Camera3D/InterfaceShapeCast")]
    pub interface_shape_cast: OnReady<Gd<ShapeCast3D>>,
    pub interface_act_react: Option<Gd<ActReactResource>>,
    #[export]
    pub head: Option<Gd<Node3D>>,
    #[export]
    pub camera: Option<Gd<Camera3D>>,
    #[export]
    pub character_controller: Option<Gd<CharacterController3D>>,
    #[export]
    pub mouse_sensitivity: f32,
    #[export]
    pub roll_speed: f32,
    #[export]
    pub max_roll: f32,
    #[export]
    pub immersion_scale: Vector3,
    #[export]
    bob_frequency: Vector3,
    original_camera_pos: Vector3,
    camera_data: CameraData,
    base: Base<Node>
}

impl PlayerCameraController3D {
    fn engage_mouselook(&mut self) {
        if Input::singleton().get_mouse_mode() == MouseMode::CAPTURED {
            GameSys::singleton().emit_signal("hud_visibility_changed".into(), &[false.to_variant()]);
            Input::singleton().set_mouse_mode(MouseMode::VISIBLE);
        } else if Input::singleton().get_mouse_mode() == MouseMode::VISIBLE {
            GameSys::singleton().emit_signal("hud_visibility_changed".into(), &[true.to_variant()]);
            Input::singleton().set_mouse_mode(MouseMode::CAPTURED);
        }
    }

    fn perform_mouse_rotation(&mut self, delta: f32) {
        self.camera_data.rotation_origins.x = (self.camera_data.rotation_origins.x - (self.camera_data.mouse_movement.y * self.mouse_sensitivity * delta)).clamp(-FRAC_PI_2, FRAC_PI_2);
        self.camera_data.target_rotation_change_y += -self.camera_data.mouse_movement.x * delta * self.mouse_sensitivity;
        let Some(char) = self.character_controller.as_mut() else {return;};
        self.camera_data.target_rotation_head.x = self.camera_data.target_rotation_head.x.lerp(self.camera_data.rotation_origins.x, 5.0 * delta);
        let rot_change_y = 0.0.lerp(self.camera_data.target_rotation_change_y, delta * 5.0);
        self.camera_data.target_rotation_change_y -= rot_change_y;
        char.rotate_y(rot_change_y);
        self.camera_data.mouse_movement = Vector2::ZERO;
    }

    fn tilt_camera(&mut self, delta: f32) {
        let Some(char) = self.character_controller.as_mut() else {return;};
        let char_x_axis: Vector3 = -char.get_global_transform().basis.col_a();
        let rot_dot = char
            .bind()
            .movement_data
            .as_ref()
            .map(|m|
                if m.velocity.is_zero_approx() {
                    Vector3::ZERO
                } else {
                    m.velocity.normalized()
                })
            .unwrap_or(Vector3::ZERO)
            .dot(char_x_axis);
        if rot_dot.abs() > self.roll_speed {
            self.camera_data.target_rotation_head.z = self.camera_data.target_rotation_head.z.lerp(self.max_roll * rot_dot.sign(), 2.0 * delta);
        } else {
            self.camera_data.target_rotation_head.z = self.camera_data.target_rotation_head.z.lerp(0.0, 2.0 * delta);
        }
    }

    fn get_and_update_bob(&mut self, delta: f32, frequency_mod: f32, axis: usize) -> f32 {
        let is_moving = self.character_controller.as_ref().map(
            |c| c.bind().movement_data.as_ref().map(|md| !md.velocity.is_zero_approx())
        ).unwrap_or(None).unwrap_or(false);
        if !is_moving {
            let mut to: f64 = 0.0;
            // finish the cycle
            if self.camera_data.bobtimes[axis] > PI as f32 {
                to = 2.0 * PI;
            }
            self.camera_data.bobtimes[axis] = lerpf(self.camera_data.bobtimes[axis] as f64, to, (delta * frequency_mod * 0.5) as f64) as f32;
            return sin(self.camera_data.bobtimes[axis] as f64) as f32
        }
        self.camera_data.bobtimes[axis] = fmod((self.camera_data.bobtimes[axis] + delta * frequency_mod) as f64, 2.0 * PI) as f32;
        sin(self.camera_data.bobtimes[axis] as f64) as f32
    }

    fn calculate_bob(&mut self, delta: f32) -> Vector3 {
        Vector3::new(
            self.get_and_update_bob(delta, self.bob_frequency.x, 0),
            self.get_and_update_bob(delta, self.bob_frequency.y, 1),
            self.get_and_update_bob(delta, self.bob_frequency.z, 2),
        )
    }
}


#[godot_api]
impl INode for PlayerCameraController3D {

    fn physics_process(&mut self, delta: f64) {
        if Input::singleton().is_action_just_pressed("mouselook".into()) {
            self.engage_mouselook();
        }
        self.perform_mouse_rotation(delta as f32);
        self.tilt_camera(delta as f32);
        let bob = self.calculate_bob(delta as f32);
        if let Some(head) = self.head.as_mut() {
            head.set_rotation(self.camera_data.target_rotation_head);
        }
        if let Some(camera) = self.camera.as_mut() {
            camera.set_position(self.original_camera_pos + bob * self.immersion_scale);
        }
        if Input::singleton().is_action_just_pressed("frob".into()) {
            let actor = self.base().clone();
            let Some(acts) = self.interface_act_react.as_mut() else {return;};
            if self.interface_shape_cast.is_colliding() {
                if let Some(Ok(react_area)) = self.interface_shape_cast.get_collider(0).map(|o| o.try_cast::<ActReactArea3D>()) {

                    let context = dict! {
                        "actor": actor,
                        "reactor": react_area.bind().target.clone(),

                    };
                    ActReactExecutor::singleton().bind_mut().react(acts.clone(), react_area.bind().act_react.clone().unwrap(), context);
                }

            }
        }

    }

    fn ready(&mut self) {
        self.original_camera_pos = self.camera.as_ref().map(|c| c.get_position()).unwrap_or(Vector3::ZERO);
    }


    fn input(&mut self, event: Gd<InputEvent>) {
        if Input::singleton().get_mouse_mode() == MouseMode::CAPTURED {
            let Ok(mouse_motion) = event.try_cast::<InputEventMouseMotion>() else {return;};
            self.camera_data.mouse_movement += mouse_motion.get_relative();
        }
    }

}