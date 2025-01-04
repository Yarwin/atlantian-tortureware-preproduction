use crate::act_react::stimulis::Stimuli;
use godot::prelude::*;

#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct ActCombine {
    #[export]
    pub reduce_stack: bool,
    #[export]
    pub combinator: StringName,
    #[var]
    #[init(val = Stimuli::Combine)]
    stim_type: Stimuli,
}

#[godot_api]
impl ActCombine {
    #[func]
    fn get_context(&self) -> Dictionary {
        dict! {
            "reduce_stack": self.reduce_stack,
            "combinator": self.combinator.clone()
        }
    }
}
