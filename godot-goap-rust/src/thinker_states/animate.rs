use std::time::SystemTime;
use crate::thinker_states::types::{StateArguments, ThinkerState};
use godot::classes::AnimationNodeStateMachinePlayback;
use godot::prelude::*;
use serde::{Deserialize, Serialize};
use crate::ai::working_memory::{Event, FactQuery, FactQueryCheck, WorkingMemoryFactType};
use crate::ai::working_memory::Event::AnimationCompleted;


#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub enum AnimationMode {
    #[default]
    OneShot,
    Cyclic,
    Timed(f64),
    Loops(u32)
}

#[derive(Debug)]
#[allow(unused_attributes)]
pub struct AnimateState {
    pub name: String,
    pub mode: AnimationMode,
    pub loops_performed: u32,
    pub creation_time: SystemTime
}

impl AnimateState {
    pub fn new_boxed(name: String, mode: AnimationMode) -> Box<Self> {
        Box::new(AnimateState {
            name,
            mode,
            loops_performed: 0,
            creation_time: SystemTime::now()
        })
    }

    pub fn play(&mut self, args: &mut StateArguments) {
        let mut bind = args.base.bind_mut();
        let Some(anim_tree) = bind.animation_tree.as_mut() else {
            return;
        };
        let mut anim_node_state_machine = anim_tree
            .get("parameters/playback".into())
            .to::<Gd<AnimationNodeStateMachinePlayback>>();
        anim_node_state_machine.travel(self.name.clone().into());
    }
}


impl ThinkerState for AnimateState {
    fn exit(&mut self, _args: &mut StateArguments) {}

    fn enter(&mut self, mut args: StateArguments) {
        self.play(&mut args);
    }

    fn physics_process(&mut self, _delta: f64, mut args: StateArguments) {
        let mut is_finished = false;
        match self.mode {
            // set animation to complete after getting some signal
            AnimationMode::OneShot => {
                let query = FactQuery::with_check(
                    FactQueryCheck::Match(WorkingMemoryFactType::Event(AnimationCompleted(self.name.clone()))));
                let Some(_f) = args.working_memory.find_and_mark_as_invalid(query) else {return;};
                is_finished = true;
            }
            // cyclic animations never ends
            AnimationMode::Cyclic => {}
            AnimationMode::Timed(time) => {
                if self.creation_time.elapsed().unwrap().as_secs_f64() > time {
                    is_finished = true;
                }
            }
            AnimationMode::Loops(loop_amount) => {
                let query = FactQuery::with_check(
                    FactQueryCheck::Match(WorkingMemoryFactType::Event(AnimationCompleted(self.name.clone()))));
                let Some(_f) = args.working_memory.find_and_mark_as_invalid(query) else {return;};
                self.loops_performed += 1;
                if self.loops_performed >= loop_amount {
                    is_finished = true;
                }
                self.play(&mut args);
            }
        };
        if is_finished {
            args.blackboard.animation_completed = true;
        }

    }

    fn update_animation(&mut self, _args: StateArguments) {}
}
