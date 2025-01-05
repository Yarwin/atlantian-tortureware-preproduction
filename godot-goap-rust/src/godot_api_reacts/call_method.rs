use crate::act_react::act_react_resource::Reaction;
use crate::act_react::game_effect::{EffectResult, GameEffect};
use godot::classes::Resource;
use godot::prelude::*;

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct CallMethodGameEffect {
    #[export]
    pub method_name: StringName,
    #[export]
    args: Array<Variant>,
    #[export]
    method_display: GString,
    base: Base<Resource>,
}

#[godot_dyn]
impl Reaction for CallMethodGameEffect {
    fn get_react_display(&self) -> Option<GString> {
        if self.method_display.is_empty() {
            return Some(GString::from("Use"));
        }
        Some(self.method_display.clone())
    }
    fn build_effect(
        &self,
        act_context: &Dictionary,
        context: &Dictionary,
    ) -> Option<DynGd<Object, dyn GameEffect>> {
        let Some(target) = context.get("reactor").map(|v| v.to::<Gd<Object>>()) else {
            panic!("tried to instantiate command without proper context!")
        };
        let effect = CallMethod {
            target: Some(target),
            args: self.args.clone(),
            method: self.method_name.clone(),
        };
        let obj = Gd::from_object(effect);
        Some(obj.into_dyn::<dyn GameEffect>().upcast())
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

#[godot_dyn]
impl GameEffect for CallMethod {
    fn execute(&mut self) -> EffectResult {
        self.target
            .as_mut()
            .unwrap()
            .callv(&self.method, &self.args);
        EffectResult::Free
    }
    fn revert(&mut self) -> EffectResult {
        panic!("call method can't be reversed!")
    }
}
