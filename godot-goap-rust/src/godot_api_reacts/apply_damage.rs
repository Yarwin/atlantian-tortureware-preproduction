use crate::act_react::act_react_resource::Reaction;
use crate::act_react::game_effect::{EffectResult, GameEffect};
use crate::receiver::damage_receptor_component::{DamageReceptorComponent, ReceivedDamage};
use godot::classes::Resource;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct ApplyDamageGameEffect {
    #[export(range = (0.0, 2.0))]
    #[init(val = 1.0)]
    vulnerability: f64,
    #[export(range = (0.0, 2.0))]
    #[init(val = 1.0)]
    pain_vulnerability: f64,
    #[export]
    flat_reduction: f64,
    #[export]
    flat_pain_reduction: f64,
    base: Base<Resource>,
}

#[godot_dyn]
impl Reaction for ApplyDamageGameEffect {
    fn build_effect(
        &self,
        act_context: &Dictionary,
        context: &Dictionary,
    ) -> Option<DynGd<Object, dyn GameEffect>> {
        let Some(Ok(reactor)) = context
            .get("reactor")
            .map(|v| v.try_to::<Gd<DamageReceptorComponent>>())
        else {
            return None;
        };
        let mut new = act_context.clone();
        new.extend_dictionary(context, true);
        let Ok(mut damage) = ReceivedDamage::try_from_godot(new) else {
            return None;
        };
        damage.strength = damage.strength * self.vulnerability - self.flat_reduction;
        damage.pain = damage.pain * self.pain_vulnerability - self.flat_pain_reduction;
        let effect = ApplyDamage {
            reactor: Some(reactor),
            damage: Some(damage),
        };
        let obj = Gd::from_object(effect);
        Some(obj.into_dyn::<dyn GameEffect>().upcast())
    }
}

#[derive(GodotClass)]
#[class(init, base=Object)]
pub struct ApplyDamage {
    reactor: Option<Gd<DamageReceptorComponent>>,
    damage: Option<ReceivedDamage>,
}

#[godot_dyn]
impl GameEffect for ApplyDamage {
    fn execute(&mut self) -> EffectResult {
        let mut reactor = self.reactor.take().unwrap();
        reactor
            .bind_mut()
            .resolve_damage(self.damage.take().unwrap());
        EffectResult::Free
    }
}
