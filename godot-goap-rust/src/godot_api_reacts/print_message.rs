use godot::prelude::*;
use crate::act_react::game_effect::{EffectResult, GameEffect, GameEffectProcessor};
use crate::act_react::game_effect_builder::{GameEffectInitializer, register_effect_builder};


#[derive(GodotClass, Debug)]
#[class(base=Resource)]
pub struct PrintMessageGameEffect {
    #[export]
    pub message: GString,
    base: Base<Resource>
}


#[godot_api]
impl IResource for PrintMessageGameEffect {
    fn init(base: Base<Self::Base>) -> Self {
        register_effect_builder::<Self>(Self::class_name().to_gstring());
        PrintMessageGameEffect{ message: Default::default(), base }
    }
}

impl GameEffectInitializer for PrintMessageGameEffect {
    fn build(&self, _act: &Dictionary, _context: &Dictionary) -> GameEffectProcessor {
        let print_message = PrintMessage {
            message: self.message.clone()
        };
        let obj = Gd::from_object(print_message);
        GameEffectProcessor::new(obj)
    }
}



#[derive(GodotClass, Debug)]
#[class(init, base=Object)]
pub struct PrintMessage {
    pub message: GString
}


impl GameEffect for PrintMessage {
    fn execute(&mut self) -> EffectResult {
        godot_print!("{}", self.message);
        EffectResult::Free
    }
}