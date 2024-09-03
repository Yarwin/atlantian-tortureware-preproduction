use godot::classes::Resource;
use godot::prelude::*;
use crate::act_react::game_effect::{EffectResult, GameEffect, GameEffectProcessor};
use crate::act_react::game_effect_builder::GameEffectInitializer;
use crate::receiver::damage_receptor_component::{DamageReceptorComponent, ReceivedDamage};

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct ApplyDamageGameEffect {
    #[export(range = (0.0, 2.0))]
    #[init(default = 1.0)]
    vulnerability: f64,
    #[export(range = (0.0, 2.0))]
    #[init(default = 1.0)]
    pain_vulnerability: f64,
    #[export]
    flat_reduction: f64,
    #[export]
    flat_pain_reduction: f64,
    base: Base<Resource>
}

impl GameEffectInitializer for ApplyDamageGameEffect {
    fn build(&self, act_context: &Dictionary, context: &Dictionary) -> Option<GameEffectProcessor> {

        let Some(Ok(reactor)) = context.get("reactor").map(|v| v.try_to::<Gd<DamageReceptorComponent>>()) else {return None};
        let mut new = act_context.clone();
        new.extend_dictionary(context.clone(), true);
        let Ok(mut damage) = ReceivedDamage::try_from_godot(new) else {return None};
        damage.strength = damage.strength * self.vulnerability - self.flat_reduction;
        damage.pain = damage.pain * self.pain_vulnerability - self.flat_pain_reduction;
        let effect = ApplyDamage {
            reactor: Some(reactor),
            damage: Some(damage)
        };
        let obj = Gd::from_object(effect);
        Some(GameEffectProcessor::new(obj))
    }
}


#[derive(GodotClass, Debug)]
#[class(init, base=Object)]
pub struct ApplyDamage {
    reactor: Option<Gd<DamageReceptorComponent>>,
    damage: Option<ReceivedDamage>,
}

impl GameEffect for ApplyDamage {
    fn execute(&mut self) -> EffectResult {
        let mut reactor = self.reactor.take().unwrap();
        reactor.bind_mut().resolve_damage(self.damage.take().unwrap());
        EffectResult::Free
    }
}
