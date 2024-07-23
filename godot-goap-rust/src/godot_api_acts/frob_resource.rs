use godot::prelude::*;
use crate::act_react::stimulis::Stimuli;

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct ActFrob {
    #[var]
    #[init(default = Stimuli::Frob)]
    stim_type: Stimuli,
}


#[godot_api]
impl ActFrob {
    #[func]
    fn get_context(&self) -> Dictionary {
        dict! {}
    }
}
