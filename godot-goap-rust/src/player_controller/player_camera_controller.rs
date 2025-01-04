use crate::character_controler::character_controller_3d::CharacterController3D;
use crate::godot_api::gamesys::GameSys;
use godot::classes::input::MouseMode;
use godot::classes::{InputEvent, InputEventMouseMotion, ShapeCast3D};
use godot::global::{fmod, lerpf, sin};
use godot::prelude::*;
use std::f32::consts::FRAC_PI_2;
use std::f64::consts::PI;
use std::time::SystemTime;

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
    #[export]
    #[init(val = 0.1)]
    step_damping_time: f32,
    current_bob: Vector3,
    damping_time: Option<SystemTime>,
    original_camera_pos: Vector3,
    camera_data: CameraData,
    base: Base<Node>,
}

impl PlayerCameraController3D {
    fn engage_mouselook(&mut self) {
        if Input::singleton().get_mouse_mode() == MouseMode::CAPTURED {
            GameSys::singleton().emit_signal("hud_visibility_changed", &[false.to_variant()]);
            Input::singleton().set_mouse_mode(MouseMode::VISIBLE);
        } else if Input::singleton().get_mouse_mode() == MouseMode::VISIBLE {
            GameSys::singleton().emit_signal("hud_visibility_changed", &[true.to_variant()]);
            Input::singleton().set_mouse_mode(MouseMode::CAPTURED);
        }
    }

    fn perform_mouse_rotation(&mut self, delta: f32) {
        self.camera_data.rotation_origins.x = (self.camera_data.rotation_origins.x
            - (self.camera_data.mouse_movement.y * self.mouse_sensitivity * delta))
            .clamp(-FRAC_PI_2, FRAC_PI_2);
        self.camera_data.target_rotation_change_y +=
            -self.camera_data.mouse_movement.x * delta * self.mouse_sensitivity;
        let Some(char) = self.character_controller.as_mut() else {
            return;
        };
        self.camera_data.target_rotation_head.x = self
            .camera_data
            .target_rotation_head
            .x
            .lerp(self.camera_data.rotation_origins.x, 5.0 * delta);
        let rot_change_y = 0.0.lerp(self.camera_data.target_rotation_change_y, delta * 5.0);
        self.camera_data.target_rotation_change_y -= rot_change_y;
        char.rotate_y(rot_change_y);
        self.camera_data.mouse_movement = Vector2::ZERO;
    }

    fn tilt_camera(&mut self, delta: f32) {
        let Some(char) = self.character_controller.as_mut() else {
            return;
        };
        let char_x_axis: Vector3 = -char.get_global_transform().basis.col_a();
        let rot_dot = char
            .bind()
            .movement_data
            .as_ref()
            .map(|m| {
                if m.velocity.is_zero_approx() {
                    Vector3::ZERO
                } else {
                    m.velocity.normalized()
                }
            })
            .unwrap_or(Vector3::ZERO)
            .dot(char_x_axis);
        if rot_dot.abs() > self.roll_speed {
            self.camera_data.target_rotation_head.z = self
                .camera_data
                .target_rotation_head
                .z
                .lerp(self.max_roll * rot_dot.sign(), 2.0 * delta);
        } else {
            self.camera_data.target_rotation_head.z = self
                .camera_data
                .target_rotation_head
                .z
                .lerp(0.0, 2.0 * delta);
        }
    }

    fn get_and_update_bob(&mut self, delta: f32, frequency_mod: f32, axis: usize) -> f32 {
        let is_moving = self
            .character_controller
            .as_ref()
            .map(|c| {
                c.bind()
                    .movement_data
                    .as_ref()
                    .map(|md| !md.velocity.is_zero_approx())
            })
            .unwrap_or(None)
            .unwrap_or(false);
        if !is_moving {
            let mut to: f64 = 0.0;
            // finish the cycle
            if self.camera_data.bobtimes[axis] > PI as f32 {
                to = 2.0 * PI;
            }
            self.camera_data.bobtimes[axis] = lerpf(
                self.camera_data.bobtimes[axis] as f64,
                to,
                (delta * frequency_mod * 0.5) as f64,
            ) as f32;
            return sin(self.camera_data.bobtimes[axis] as f64) as f32;
        }
        self.camera_data.bobtimes[axis] = fmod(
            (self.camera_data.bobtimes[axis] + delta * frequency_mod) as f64,
            2.0 * PI,
        ) as f32;
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
        if Input::singleton().is_action_just_pressed("mouselook") {
            self.engage_mouselook();
        }
        self.perform_mouse_rotation(delta as f32);
        self.tilt_camera(delta as f32);
        let bob = self.calculate_bob(delta as f32);

        let Some(head) = self.head.as_mut() else {
            return;
        };
        head.set_rotation(self.camera_data.target_rotation_head);

        let Some(cam) = self.camera.as_mut() else {
            return;
        };
        let Some(damp_time) = self.damping_time.as_mut() else {
            cam.set_position(self.original_camera_pos + bob * self.immersion_scale);
            return;
        };

        // This is not a physics interpolation
        // Stepping happens in one frame
        // Thus we lag camera by a bit to prevent the teleporting
        let elapsed = damp_time.elapsed().unwrap().as_secs_f32();
        if elapsed > self.step_damping_time {
            self.damping_time = None;
            cam.set_as_top_level(false);
            cam.set_transform(Transform3D::new(
                Basis::from_cols(Vector3::RIGHT, Vector3::UP, Vector3::BACK),
                Vector3::ZERO,
            ));
        } else {
            let target_transform = head.get_global_transform();
            let mut transform = Transform3D::new(target_transform.basis, Vector3::ZERO);
            let prev_transform = cam.get_transform();
            let diff = (target_transform.origin
                + self.original_camera_pos
                + self.current_bob * self.immersion_scale)
                - prev_transform.origin;
            let lerp_speed = (elapsed / 0.1).max(0.5);
            transform.origin = prev_transform.origin + diff * lerp_speed;
            cam.set_transform(transform);
        }
    }

    fn ready(&mut self) {
        self.original_camera_pos = self
            .camera
            .as_ref()
            .map(|c| c.get_position())
            .unwrap_or(Vector3::ZERO);
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if Input::singleton().get_mouse_mode() == MouseMode::CAPTURED {
            let Ok(mouse_motion) = event.try_cast::<InputEventMouseMotion>() else {
                return;
            };
            self.camera_data.mouse_movement += mouse_motion.get_relative();
        }
    }
}

#[godot_api]
impl PlayerCameraController3D {
    #[func]
    fn _on_step(&mut self, step_height: Vector3) {
        if self.damping_time.is_some() {
            self.damping_time = Some(SystemTime::now());
            return;
        }
        if let Some(cam) = self.camera.as_mut() {
            let cam_pos = cam.get_position();
            cam.set_as_top_level(true);
            cam.set_global_position(
                self.head.as_ref().unwrap().get_global_position() + cam_pos - step_height,
            );
            self.damping_time = Some(SystemTime::now());
        }
    }
}
