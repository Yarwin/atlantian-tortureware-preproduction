use crate::act_react::act_react_resource::Emitter;
use crate::act_react::stimulis::Stimuli;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct ActPlayerFrob {
    #[var]
    #[init(val = Stimuli::PlayerFrob)]
    stim_type: Stimuli,
}

#[godot_dyn]
impl Emitter for ActPlayerFrob {
    fn get_stim_type(&self) -> Stimuli {
        Stimuli::PlayerFrob
    }

    fn get_context(&self) -> Dictionary {
        dict! {}
    }
}
