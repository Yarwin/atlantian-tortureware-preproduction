use crate::act_react::act_react_resource::Emitter;
use crate::act_react::stimulis::Stimuli;
use godot::builtin::{dict, Dictionary};
use godot::prelude::*;

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct ActPressure {
    #[var]
    #[init(val = Stimuli::Pressure)]
    stim_type: Stimuli,
}

#[godot_dyn]
impl Emitter for ActPressure {
    fn get_stim_type(&self) -> Stimuli {
        Stimuli::Pressure
    }

    fn get_context(&self) -> Dictionary {
        dict! {}
    }
}
