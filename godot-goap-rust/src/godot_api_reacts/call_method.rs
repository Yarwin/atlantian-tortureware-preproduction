use godot::classes::Resource;
use godot::prelude::*;
use crate::act_react::game_effect::{EffectResult, GameEffect, GameEffectProcessor};
use crate::act_react::game_effect_builder::{GameEffectInitializer, register_effect_builder};


#[derive(GodotClass, Debug)]
#[class(base=Resource)]
pub struct CallMethodGameEffect {
    #[export]
    pub method_name: StringName,
    #[export]
    args: Array<Variant>,
    base: Base<Resource>
}

#[godot_api]
impl IResource for CallMethodGameEffect {
    fn init(base: Base<Self::Base>) -> Self {
        register_effect_builder::<Self>(Self::class_name().to_gstring());
        CallMethodGameEffect { method_name: Default::default(), args: array![], base }
    }
}

impl GameEffectInitializer for CallMethodGameEffect {
    fn build(&self, _act: &Dictionary, context: &Dictionary) -> GameEffectProcessor {
        let Some(target) = context.get("reactor").map(|v| v.to::<Gd<Object>>()) else {panic!("tried to instantiate command without proper context!")};
        let effect = CallMethod {
            target: Some(target),
            args: self.args.clone(),
            method: self.method_name.clone()
        };
        let obj = Gd::from_object(effect);
        GameEffectProcessor::new(obj)
    }
}

#[derive(GodotClass, Debug)]
#[class(init, base=Object)]
pub struct CallMethod {
    // wrapped in Option for init
    target: Option<Gd<Object>>,
    args: Array<Variant>,
    method: StringName,
}


impl GameEffect for CallMethod {
    fn execute(&mut self) -> EffectResult {
        self.target.as_mut().unwrap().callv(self.method.clone(), self.args.clone());
        EffectResult::Free
    }
    fn revert(&mut self) -> EffectResult {
        panic!("call method can't be reversed!")
    }
}