use crate::act_react::act_react_resource::{ActReactResource, Emitter};
use crate::act_react::game_effect::{EffectResult, GameEffect};
use crate::godot_api::gamesys::{GameSys, GameSystem};
use crate::godot_api::{CONNECT_DEFERRED, CONNECT_ONE_SHOT};
use godot::prelude::*;
use std::collections::{HashMap, VecDeque};

/// An entity responsible for creating, managing and executing commands
/// Effects are being processed & executed at the end of every physics frame.
#[derive(GodotClass)]
#[class(init, base=Object)]
pub struct ActReactExecutor {
    #[init(val = Some(VecDeque::new()))]
    to_execute: Option<VecDeque<DynGd<Object, dyn GameEffect>>>,
    to_revert: HashMap<InstanceId, DynGd<Object, dyn GameEffect>>,
    pub base: Base<Object>,
}

#[godot_api]
impl ActReactExecutor {
    #[func]
    pub fn react(
        &mut self,
        mut actor: Gd<ActReactResource>,
        mut reactor: Gd<ActReactResource>,
        context: Dictionary,
    ) {
        let mut actor_bind = actor.bind_mut();
        let mut reactor_bind = reactor.bind_mut();
        for mut meta in reactor_bind.metaproperties.iter_shared() {
            let mut meta_bind = meta.bind_mut();
            self.create_effects(&mut actor_bind, &mut meta_bind, &context);
        }
        self.create_effects(&mut actor_bind, &mut reactor_bind, &context);
    }

    #[func]
    pub fn react_single(
        &mut self,
        act: DynGd<Resource, dyn Emitter>,
        mut reactor: Gd<ActReactResource>,
        context: Dictionary,
    ) {
        let mut reactor_bind = reactor.bind_mut();
        self.create_reacts_for_act(act, &mut reactor_bind, &context)
    }

    #[func]
    pub fn process_effects(&mut self) {
        // utterly dumb hack to satisfy the compiler
        let Some(mut to_execute) = self.to_execute.take() else {
            panic!("no event queue!")
        };
        for mut effect in to_execute.drain(..) {
            let execution_effect = effect.dyn_bind_mut().execute();
            match execution_effect {
                EffectResult::Free => {
                    effect.free();
                }
                EffectResult::Revert(after) => {
                    self.to_revert.insert(effect.instance_id(), effect);
                    let mut timer = GameSys::singleton()
                        .get_tree()
                        .unwrap()
                        .create_timer(after)
                        .unwrap();
                    let callable = Callable::from_object_method(&(self.base().clone()), "revert");
                    timer
                        .connect_ex("timeout", &callable)
                        .flags(CONNECT_ONE_SHOT + CONNECT_DEFERRED)
                        .done();
                }
                // shouldn't it return an error or something???
                EffectResult::Failed => {
                    godot_print!("effect failed");
                    effect.free();
                }
            }
        }
        // put deque back where it belongs
        self.to_execute = Some(to_execute);
    }

    #[func]
    fn revert(&mut self, effect_id: InstanceId) {
        // bail if no effect to revert (shouldn't it panic instead?)
        let Some(mut effect) = self.to_revert.remove(&effect_id) else {
            return;
        };
        effect.dyn_bind_mut().revert();
        effect.free();
    }
}

impl ActReactExecutor {
    pub fn add_effect(&mut self, effect: DynGd<Object, dyn GameEffect>) {
        let to_execute = self
            .to_execute
            .as_mut()
            .expect("no deque of effects to execute!");
        to_execute.push_back(effect);
    }

    fn create_reacts_for_act(
        &mut self,
        act: DynGd<Resource, dyn Emitter>,
        reactor: &mut GdMut<ActReactResource>,
        context: &Dictionary,
    ) {
        let stimuli = act.dyn_bind().get_stim_type();
        let act_context = act.dyn_bind().get_context();
        for react in reactor[stimuli].iter_shared() {
            if let Some(e) = react.dyn_bind().build_effect(&act_context, context) {
                self.add_effect(e);
            }
        }
    }

    pub fn create_effects(
        &mut self,
        actor: &mut GdMut<ActReactResource>,
        reactor: &mut GdMut<ActReactResource>,
        context: &Dictionary,
    ) {
        for meta in actor.metaproperties.iter_shared() {
            for act in meta.bind().emits.iter_shared() {
                self.create_reacts_for_act(act, reactor, context);
            }
        }
        for act in actor.emits.iter_shared() {
            self.create_reacts_for_act(act, reactor, context);
        }
    }
}

impl GameSystem for ActReactExecutor {
    const NAME: &'static str = "ActReactExecutor";
    fn physics_process(&mut self, _delta: f64) {
        // process&apply all the effects at the end of the current frame
        self.base_mut().call_deferred("process_effects", &[]);
    }
}
