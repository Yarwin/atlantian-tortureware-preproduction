use godot::classes::{Engine, IRigidBody3D, PhysicsDirectBodyState3D, RigidBody3D, Tween};
use godot::classes::tween::TransitionType;
use godot::prelude::*;
use crate::act_react::act_react_resource::ActReactResource;
use crate::act_react::react_area_3d::ActReactArea3D;

/// a world object that can be kicked, grabbed, threw (?) and moved around
#[derive(GodotClass)]
#[class(init, base=RigidBody3D)]
pub struct WorldObject {
    #[var]
    #[init(default = false)]
    pub is_contact_velocity_achieved: bool,
    #[var]
    pub is_grabbed: bool,
    #[export]
    contact_velocity: f32,
    #[export]
    pub contact_velocity_effects: Option<Gd<ActReactResource>>,
    #[export]
    pub act_react_area: Option<Gd<ActReactArea3D>>,
    #[export]
    pub mesh: Option<Gd<Node3D>>,
    pub tween: Option<Gd<Tween>>,
    base: Base<RigidBody3D>
}

#[godot_api]
impl WorldObject {
    #[signal]
    fn grabbed();
    #[signal]
    fn released();

    #[signal]
    fn contact_velocity_achieved();

    #[signal]
    fn contact_velocity_left();

    #[func]
    fn grab(&mut self) {
        if self.is_grabbed {return;}
        self.is_contact_velocity_achieved = false;
        self.base_mut().set_lock_rotation_enabled(true);
        self.is_grabbed = true;
        self.base_mut().emit_signal("grabbed".into(), &[]);
    }

    #[func]
    fn release(&mut self) {
        if !self.is_grabbed {return;}
        self.is_grabbed = false;
        self.base_mut().set_lock_rotation_enabled(false);
        self.base_mut().emit_signal("released".into(), &[]);
    }
    #[func]
    fn make_transparent_default(&mut self) {
        self.make_transparent();
    }

    #[func(virtual)]
    fn make_transparent(&mut self) {
        if let Some(mesh) = self.mesh.as_ref() {
            if let Some(mut tween) = self.tween.take() {
                tween.kill();
            }
            let mut tween = self.base().get_tree().unwrap().create_tween();
            tween.unwrap().tween_property(mesh, "transparency".into(), 0.15.to_variant(), 1.0).unwrap().set_trans(TransitionType::EXPO);;
        }
    }

    #[func]
    fn untransparent_default(&mut self) {
        self.untransparent();
    }

    #[func(virtual)]
    fn untransparent(&mut self) {
        if let Some(mesh) = self.mesh.as_ref() {
            if let Some(mut tween) = self.tween.take() {
                tween.kill();
            }
            let mut tween = self.base().get_tree().unwrap().create_tween();
            tween.unwrap().tween_property(mesh, "transparency".into(), 0.0.to_variant(), 1.0).unwrap().set_trans(TransitionType::EXPO);
        }
    }
}

#[godot_api]
impl IRigidBody3D for WorldObject {
    fn integrate_forces(&mut self, state: Gd<PhysicsDirectBodyState3D>) {
        // don't propagate events if grabbed
        if self.is_grabbed {return;}
        if !self.is_contact_velocity_achieved && state.get_linear_velocity().length() > self.contact_velocity {
            self.is_contact_velocity_achieved = true;
            self.base_mut().emit_signal("contact_velocity_achieved".into(), &[]);
        } else if self.is_contact_velocity_achieved && state.get_linear_velocity().length() < self.contact_velocity {
            self.is_contact_velocity_achieved = false;
            self.base_mut().emit_signal("contact_velocity_left".into(), &[]);
        }

    }
}
