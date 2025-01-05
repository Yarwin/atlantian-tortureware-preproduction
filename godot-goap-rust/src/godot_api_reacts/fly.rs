use crate::act_react::act_react_resource::Reaction;
use crate::act_react::game_effect::{EffectResult, GameEffect};
use godot::classes::RigidBody3D;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub(crate) struct FlyGameEffect {
    #[export]
    #[init(val = 4.0)]
    pub force: f32,
    base: Base<Resource>,
}

impl Reaction for FlyGameEffect {
    fn build_effect(
        &self,
        act_context: &Dictionary,
        context: &Dictionary,
    ) -> Option<DynGd<Object, dyn GameEffect>> {
        let Some(flier) = context.get("reactor").map(|v| v.to::<Gd<RigidBody3D>>()) else {
            panic!("no reactor!")
        };
        let force = self.force;
        let force_multiplier = act_context
            .get("force")
            .map(|v| v.try_to::<f32>().unwrap_or(1.0))
            .unwrap_or(1.0);
        let Some(direction) = context.get("direction").map(|v| -(v.to::<Vector3>())) else {
            panic!("no direction")
        };
        let mut at = Vector3::ZERO;
        if let Some(Ok(position)) = context.get("position").map(|v| v.try_to::<Vector3>()) {
            at = flier.to_local(position);
        }
        let effect = Fly {
            target: Some(flier),
            force: force * force_multiplier,
            at,
            direction,
        };
        let obj = Gd::from_object(effect);
        Some(obj.into_dyn::<dyn GameEffect>().upcast())
    }
}

#[derive(GodotClass)]
#[class(init, base=Object)]
pub struct Fly {
    target: Option<Gd<RigidBody3D>>,
    force: f32,
    at: Vector3,
    direction: Vector3,
}

#[godot_dyn]
impl GameEffect for Fly {
    fn execute(&mut self) -> EffectResult {
        self.target
            .take()
            .unwrap()
            .apply_impulse_ex(self.direction * self.force)
            .position(self.at)
            .done();
        EffectResult::Free
    }
}
