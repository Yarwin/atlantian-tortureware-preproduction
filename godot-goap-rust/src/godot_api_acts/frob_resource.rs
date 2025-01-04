use crate::act_react::stimulis::Stimuli;
use godot::prelude::*;

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct ActFrob {
    #[var]
    #[init(val = Stimuli::Frob)]
    stim_type: Stimuli,
}

#[godot_api]
impl ActFrob {
    #[func]
    fn get_context(&self) -> Dictionary {
        dict! {}
    }
}
