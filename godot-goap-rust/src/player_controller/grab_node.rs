use godot::classes::{Marker3D, RigidBody3D};
use godot::prelude::*;

/// a node responsible for keeping attached rigidbody in target position.
#[derive(GodotClass)]
#[class(init, base=Node3D)]
pub struct GrabNode {
    #[var]
    pub attached: Option<Gd<RigidBody3D>>,
    #[init(node = "Marker3D")]
    grab_pos: OnReady<Gd<Marker3D>>,
    #[export]
    #[init (default = 10.)]
    carrying_velocity_mul: f32,
    base: Base<Node3D>
}

#[godot_api]
impl INode3D for GrabNode {
    fn physics_process(&mut self, _delta: f64) {
        if let Some(attached) = self.attached.as_mut() {
            let pos = attached.get_global_position();
            let v = (self.grab_pos.get_global_position() - pos) * self.carrying_velocity_mul;

            if attached.has_method("set_grab_linear_velocity".into()) {
                attached.call("set_grab_linear_velocity".into(), &[v.to_variant()]);
            } else {
                attached.set_linear_velocity(v);
            }

            attached.set_global_rotation(self.grab_pos.get_global_rotation() * Vector3::UP);
        }

    }
}

#[godot_api]
impl GrabNode {
    #[signal]
    fn object_grabbed(object: Gd<RigidBody3D>);
    #[signal]
    fn object_released();

    #[func]
    pub fn attach_rigid(&mut self, mut object: Gd<RigidBody3D>) {
        if self.attached.is_some() { return; }
        if object.has_method("make_transparent".into()) {
            object.call("make_transparent".into(), &[]);
        } else if object.has_method("make_transparent_default".into()) {
            object.call("make_transparent_default".into(), &[]);
        }
        if object.has_method("grab".into()) {
            object.call("grab".into(), &[]);
        }
        self.base_mut().emit_signal("object_grabbed".into(), &[object.to_variant()]);
        self.attached = Some(object);
    }
    #[func]
    pub fn detach(&mut self) {
        if let Some(mut grabbed) = self.attached.take() {
            if grabbed.has_method("untransparent".into()) {
                grabbed.call("untransparent".into(), &[]);
            } else if grabbed.has_method("untransparent_default".into()) {
                grabbed.call("untransparent_default".into(), &[]);
            }
            if grabbed.has_method("release".into()) {
                grabbed.call("release".into(), &[]);
            }
        }
        self.base_mut().emit_signal("object_released".into(), &[]);
    }
}
