use godot::classes::Resource;
use godot::prelude::*;
use crate::act_react::game_effect::{EffectResult, GameEffect, GameEffectProcessor};
use crate::act_react::game_effect_builder::GameEffectInitializer;


#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct CallMethodGameEffect {
    #[export]
    pub method_name: StringName,
    #[export]
    args: Array<Variant>,
    #[export]
    method_display: GString,
    base: Base<Resource>
}

#[godot_api]
impl CallMethodGameEffect {
    #[func]
    fn get_react_display(&self) -> GString {
        if self.method_display.is_empty() {
            return GString::from("Use");
        }
        self.method_display.clone()
    }
}

impl GameEffectInitializer for CallMethodGameEffect {
    fn build(&self, _act_context: &Dictionary, context: &Dictionary) -> Option<GameEffectProcessor> {
        let Some(target) = context.get("reactor").map(|v| v.to::<Gd<Object>>()) else {panic!("tried to instantiate command without proper context!")};
        let effect = CallMethod {
            target: Some(target),
            args: self.args.clone(),
            method: self.method_name.clone()
        };
        let obj = Gd::from_object(effect);
        Some(GameEffectProcessor::new(obj))
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