use crate::act_react::act_react_resource::Emitter;
use crate::act_react::stimulis::Stimuli;
use godot::builtin::{dict, Dictionary};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct ActPlayerPressure {
    #[var]
    #[init(val = Stimuli::PlayerPressure)]
    stim_type: Stimuli,
}

#[godot_dyn]
impl Emitter for ActPlayerPressure {
    fn get_stim_type(&self) -> Stimuli {
        Stimuli::PlayerPressure
    }

    fn get_context(&self) -> Dictionary {
        dict! {}
    }
}
