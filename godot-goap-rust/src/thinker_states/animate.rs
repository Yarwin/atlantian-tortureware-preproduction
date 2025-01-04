use crate::ai::working_memory::Event::AnimationCompleted;
use crate::ai::working_memory::{FactQuery, FactQueryCheck, WMProperty};
use crate::thinker_states::types::{StateArguments, ThinkerState};
use godot::classes::AnimationNodeStateMachinePlayback;
use godot::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::SystemTime;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub enum AnimationMode {
    // play & await signal
    #[default]
    OneShot,
    // play & repeat at indefinitely
    Cyclic,
    // play for until n seconds pass
    Timed(f64),
    // play n loops
    Loops(u32),
    // sequence of one-shots to play
    Sequence {
        tree_names: VecDeque<String>,
        step: usize,
    },
}

#[derive(Debug)]
#[allow(unused_attributes)]
pub struct AnimateState {
    pub tree_name: String,
    pub name: String,
    pub mode: AnimationMode,
    pub loops_performed: u32,
    pub creation_time: SystemTime,
}

impl AnimateState {
    pub fn new_boxed(tree_name: String, name: String, mode: AnimationMode) -> Box<Self> {
        Box::new(AnimateState {
            tree_name,
            name,
            mode,
            loops_performed: 0,
            creation_time: SystemTime::now(),
        })
    }

    pub fn play(&mut self, args: &mut StateArguments) {
        let mut bind = args.base.bind_mut();
        let Some(anim_tree) = bind.animation_tree.as_mut() else {
            return;
        };
        let mut anim_node_state_machine = anim_tree
            .get("parameters/playback")
            .to::<Gd<AnimationNodeStateMachinePlayback>>();
        anim_node_state_machine.travel(&self.tree_name.clone());
    }
}

impl ThinkerState for AnimateState {
    fn exit(&mut self, _args: &mut StateArguments) {}

    fn enter(&mut self, mut args: StateArguments) {
        self.play(&mut args);
    }

    fn physics_process(&mut self, _delta: f64, mut args: StateArguments) {
        let mut is_finished = false;
        match &mut self.mode {
            // set animation to complete after getting some signal
            AnimationMode::OneShot => {
                let query = FactQuery::with_check(FactQueryCheck::Match(WMProperty::Event(
                    AnimationCompleted(self.name.clone()),
                )));
                let Some(_f) = args.working_memory.find_and_mark_as_invalid(query) else {
                    return;
                };
                is_finished = true;
            }
            AnimationMode::Cyclic => {
                // repeat if not looped in animation player
                let query = FactQuery::with_check(FactQueryCheck::Match(WMProperty::Event(
                    AnimationCompleted(self.name.clone()),
                )));
                let Some(_f) = args.working_memory.find_and_mark_as_invalid(query) else {
                    return;
                };
                self.play(&mut args);
            }
            AnimationMode::Timed(time) => {
                if self.creation_time.elapsed().unwrap().as_secs_f64() > *time {
                    is_finished = true;
                }
            }
            AnimationMode::Loops(loop_amount) => {
                let query = FactQuery::with_check(FactQueryCheck::Match(WMProperty::Event(
                    AnimationCompleted(self.tree_name.clone()),
                )));
                let Some(_f) = args.working_memory.find_and_mark_as_invalid(query) else {
                    return;
                };
                self.loops_performed += 1;
                if self.loops_performed >= *loop_amount {
                    is_finished = true;
                }
                self.play(&mut args);
            }
            AnimationMode::Sequence { tree_names, step } => {
                let query = FactQuery::with_check(FactQueryCheck::Match(WMProperty::Event(
                    AnimationCompleted(tree_names[*step].clone()),
                )));
                let Some(_f) = args.working_memory.find_and_mark_as_invalid(query) else {
                    return;
                };
                *step += 1;
                self.tree_name = tree_names.pop_front().unwrap();
                self.play(&mut args);
            }
        };
        if is_finished {
            args.blackboard.animation_completed = true;
        }
    }

    fn update_animation(&mut self, _args: StateArguments) {}
}
