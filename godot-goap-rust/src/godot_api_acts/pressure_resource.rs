use crate::act_react::stimulis::Stimuli;
use godot::builtin::{dict, Dictionary};
use godot::prelude::{godot_api, GodotClass};

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct ActPressure {
    #[var]
    #[init(val = Stimuli::Pressure)]
    stim_type: Stimuli,
}

#[godot_api]
impl ActPressure {
    #[func]
    fn get_context(&self) -> Dictionary {
        dict! {}
    }
}
