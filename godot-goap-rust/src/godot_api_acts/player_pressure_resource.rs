use crate::act_react::stimulis::Stimuli;
use godot::builtin::{dict, Dictionary};
use godot::prelude::{godot_api, GodotClass};

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct ActPlayerPressure {
    #[var]
    #[init(val = Stimuli::PlayerPressure)]
    stim_type: Stimuli,
}

#[godot_api]
impl ActPlayerPressure {
    #[func]
    fn get_context(&self) -> Dictionary {
        dict! {}
    }
}
