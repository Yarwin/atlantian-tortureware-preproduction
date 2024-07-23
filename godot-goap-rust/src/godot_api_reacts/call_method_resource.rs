use godot::classes::Resource;
use godot::prelude::*;
use crate::act_react::game_effect::{GameEffect, GameEffectProcessor};
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
        register_effect_builder::<Self>("CallMethodGameEffect".into());
        CallMethodGameEffect { method_name: Default::default(), args: array![], base }
    }
}

#[godot_api]
impl CallMethodGameEffect {
    #[func]
    pub fn builder_name(&self) -> StringName {
        "CallMethodGameEffect".into()
    }
}

impl GameEffectInitializer for CallMethodGameEffect {
    fn build(&self, _act: &Dictionary, context: &Dictionary) -> GameEffectProcessor {
        let Some(target) = context.get("reactor").map(|v| v.to::<Gd<Object>>()) else {panic!("tried to instantiate command without proper context!")};
        let effect = CallMethod {
            target,
            args: self.args.clone(),
            method: self.method_name.clone()
        };
        let obj = Gd::from_object(effect);
        GameEffectProcessor::new(obj)
    }
}

#[derive(GodotClass, Debug)]
#[class(no_init, base=Object)]
pub struct CallMethod {
    target: Gd<Object>,
    args: Array<Variant>,
    method: StringName,
}


impl GameEffect for CallMethod {
    fn execute(&mut self) {
        self.target.callv(self.method.clone(), self.args.clone());
    }
}