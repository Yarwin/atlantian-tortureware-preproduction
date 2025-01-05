use crate::act_react::act_react_resource::Reaction;
use crate::act_react::game_effect::{EffectResult, GameEffect};
use godot::prelude::*;

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct PrintMessageGameEffect {
    #[export]
    pub message: GString,
    base: Base<Resource>,
}

impl Reaction for PrintMessageGameEffect {
    fn build_effect(
        &self,
        _act_context: &Dictionary,
        _context: &Dictionary,
    ) -> Option<DynGd<Object, dyn GameEffect>> {
        let print_message = PrintMessage {
            message: self.message.clone(),
        };
        let obj = Gd::from_object(print_message);
        Some(obj.into_dyn::<dyn GameEffect>().upcast())
    }
}

#[derive(GodotClass, Debug)]
#[class(init, base=Object)]
pub struct PrintMessage {
    pub message: GString,
}

#[godot_dyn]
impl GameEffect for PrintMessage {
    fn execute(&mut self) -> EffectResult {
        godot_print!("{}", self.message);
        EffectResult::Free
    }
}
