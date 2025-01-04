use crate::act_react::stimulis::Stimuli;
use godot::prelude::*;

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct ActPlayerFrob {
    #[var]
    #[init(val = Stimuli::PlayerFrob)]
    stim_type: Stimuli,
}

#[godot_api]
impl ActPlayerFrob {
    #[func]
    fn get_context(&self) -> Dictionary {
        dict! {}
    }
}
