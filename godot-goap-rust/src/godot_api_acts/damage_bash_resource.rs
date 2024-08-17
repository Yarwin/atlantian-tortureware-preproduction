use godot::prelude::*;
use crate::act_react::stimulis::Stimuli;

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct ActDamageBash {
    #[var]
    #[init(default = Stimuli::DamageBash)]
    stim_type: Stimuli,
    #[export]
    pub strength: f32,
    /// determines strength of the force applied to world objects
    #[export]
    #[init(default = 1.0)]
    pub force: f32
}


#[godot_api]
impl ActDamageBash {
    #[func]
    fn get_context(&self) -> Dictionary {
        dict! {
            "strength": self.strength,
            "force": self.force
        }
    }
}
