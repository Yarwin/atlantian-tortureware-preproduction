use std::collections::{HashMap, VecDeque};
use godot::classes::Engine;
use godot::prelude::*;
use crate::act_react::act_react_resource::ActReactResource;
use crate::act_react::game_effect::{EffectResult, GameEffect, GameEffectProcessor};
use crate::act_react::game_effect_builder::effects_registry;
use crate::act_react::stimulis::Stimuli;
use crate::godot_api::ai_manager::GodotAIManager;
use crate::godot_api::{CONNECT_DEFERRED, CONNECT_ONE_SHOT};


/// An entity responsible for creating, managing and executing commands
/// Effects are being processed & executed at the end of every physics frame.
///
/// Exported (for now on) both as autoload and engine singleton
/// for easy access both from Gdscript and Gdextension library.
///
/// In the future it might be moved to some kind of GameManager autoload.
#[derive(GodotClass)]
#[class(base=Node)]
pub struct ActReactExecutor {
    to_execute: Option<VecDeque<GameEffectProcessor>>,
    to_revert: HashMap<InstanceId, GameEffectProcessor>,
    pub base: Base<Node>,
}

#[godot_api]
impl INode for ActReactExecutor {
    fn init(base: Base<Self::Base>) -> Self {
        ActReactExecutor {
            to_execute: Some(VecDeque::new()),
            to_revert: Default::default(),
            base,
        }
    }
    fn physics_process(&mut self, _delta: f64) {
        // process&apply all the effects at the end of the current frame
        self.base_mut().call_deferred("process_effects".into(), &[]);
    }

    fn enter_tree(&mut self) {
        Engine::singleton()
            .register_singleton(ActReactExecutor::name(), self.base().clone().upcast::<Object>());
    }

    fn exit_tree(&mut self) {
        Engine::singleton().unregister_singleton(ActReactExecutor::name());
    }
}

#[godot_api]
impl ActReactExecutor {

    #[func]
    pub fn react(&mut self, mut actor: Gd<ActReactResource>, mut reactor: Gd<ActReactResource>, context: Dictionary) {
        let mut actor_bind = actor.bind_mut();
        let mut reactor_bind = reactor.bind_mut();
        for mut meta in reactor_bind.metaproperties.iter_shared() {
            let mut meta_bind = meta.bind_mut();
            self.create_effects(&mut actor_bind, &mut meta_bind, &context);
        }
        self.create_effects(&mut actor_bind, &mut reactor_bind, &context);
    }

    #[func]
    pub fn process_effects(&mut self) {
        // utterly dumb hack to satisfy the compiler
        let Some(mut to_execute) = self.to_execute.take() else {return;};
        for mut effect in to_execute.drain(..) {
            match effect.execute() {
                EffectResult::Free => {
                    effect.free();
                }
                EffectResult::Revert(after) => {
                    self.to_revert.insert(effect.instance_id(), effect);
                    let mut timer = self.base().get_tree().unwrap().create_timer(after).unwrap();
                    let callable = Callable::from_object_method(&(self.base().clone()), "revert");
                    timer.connect_ex("timeout".into(), callable).flags(CONNECT_ONE_SHOT + CONNECT_DEFERRED).done();
                }
                EffectResult::Failed => {}
            }
        }
        // put deque back where it belongs
        self.to_execute = Some(to_execute);
    }

    #[func]
    fn revert(&mut self, effect_id: InstanceId) {
        // bail if no effect to revert (shouldn't it panic instead?)
        let Some(mut effect) = self.to_revert.remove(&effect_id) else {return;};
        effect.revert();
        effect.free();
    }
}

impl ActReactExecutor {
    fn name() -> StringName {
        StringName::from("ActReactExecutor")
    }

    pub fn singleton() -> Gd<Self> {
        Engine::singleton()
            .get_singleton(ActReactExecutor::name())
            .unwrap()
            .cast::<ActReactExecutor>()
    }

    fn add_effect(&mut self, effect: GameEffectProcessor) {
        let mut to_execute = self.to_execute.as_mut().unwrap();
        to_execute.push_back(effect);
    }

    pub fn create_effects(&mut self, actor: &mut GdMut<ActReactResource>, reactor: &mut GdMut<ActReactResource>, context: &Dictionary) {
        for mut act in actor.emits.iter_shared() {
            let stimuli: Stimuli = act.get("stim_type".into()).to::<Stimuli>();
            let act_context = act.call("get_context".into(), &[]).to::<Dictionary>();

            for mut react in reactor[stimuli].iter_shared() {
                let command_init_fn = effects_registry()[&react.call("builder_name".into(), &[]).to::<StringName>()];
                let effect = (command_init_fn)(react.clone(), &act_context, context, |effect, a_context, world_context |
                    {
                        effect.build(a_context, world_context)
                    }
                );
                self.add_effect(effect);
            }
        }
    }
}