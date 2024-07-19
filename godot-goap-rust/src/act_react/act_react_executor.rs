use std::collections::VecDeque;
use godot::classes::Engine;
use godot::prelude::*;
use crate::act_react::act_react_resource::ActReactResource;
use crate::act_react::game_effect::{GameEffect, GameEffectProcessor};
use crate::act_react::game_effect_builder::effects_registry;
use crate::act_react::stimulis::Stimuli;


#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct ActReactExecutor {
    effects: VecDeque<GameEffectProcessor>,
    pub base: Base<Node>,
}

#[godot_api]
impl INode for ActReactExecutor {
    fn physics_process(&mut self, _delta: f64) {
        // process&apply all the effects at the end of the current frame
        self.base_mut().call_deferred("process_effects".into(), &[]);
    }
    fn enter_tree(&mut self) {
        Engine::singleton()
            .register_singleton("ActReactExecutor".into(), self.base().clone().upcast::<Object>());
    }

    fn exit_tree(&mut self) {
        Engine::singleton().unregister_singleton("ActReactExecutor".into());
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
            self.create_reacts(&mut actor_bind, &mut meta_bind, &context);
        }
        self.create_reacts(&mut actor_bind, &mut reactor_bind, &context);
    }

    #[func]
    pub fn process_effects(&mut self) {
        for mut effect in self.effects.drain(..) {
            effect.execute();
        }
    }
}

impl ActReactExecutor {
    fn add_effect(&mut self, effect: GameEffectProcessor) {
        self.effects.push_back(effect);
    }

    pub fn create_reacts(&mut self, actor: &mut GdMut<ActReactResource>, reactor: &mut GdMut<ActReactResource>, context: &Dictionary) {
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
                self.effects.push_back(effect);
            }
        }
    }
}