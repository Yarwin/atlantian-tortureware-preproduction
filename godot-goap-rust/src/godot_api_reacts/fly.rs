use crate::act_react::game_effect::{EffectResult, GameEffect, GameEffectProcessor};
use crate::act_react::game_effect_builder::GameEffectInitializer;
use godot::classes::RigidBody3D;
use godot::prelude::*;

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub(crate) struct FlyGameEffect {
    #[export]
    #[init(val = 4.0)]
    pub force: f32,
    base: Base<Resource>,
}

impl GameEffectInitializer for FlyGameEffect {
    fn build(&self, act_context: &Dictionary, context: &Dictionary) -> Option<GameEffectProcessor> {
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
        Some(GameEffectProcessor::new(obj))
    }
}

#[derive(GodotClass, Debug)]
#[class(init, base=Object)]
pub struct Fly {
    target: Option<Gd<RigidBody3D>>,
    force: f32,
    at: Vector3,
    direction: Vector3,
}

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
