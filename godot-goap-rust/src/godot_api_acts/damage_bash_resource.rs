use godot::prelude::*;
use crate::act_react::stimulis::Stimuli;

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct ActDamageBash {
    #[var]
    #[init(default = Stimuli::DamageBash)]
    stim_type: Stimuli,
    #[export]
    pub strength: f64
}


#[godot_api]
impl ActDamageBash {
    #[func]
    fn get_context(&self) -> Dictionary {
        dict! {
            "strength": self.strength
        }
    }
}
