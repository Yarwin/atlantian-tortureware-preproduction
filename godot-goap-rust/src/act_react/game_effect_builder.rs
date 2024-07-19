use std::collections::HashMap;
use std::fmt::Debug;
use godot::obj::Bounds;
use godot::obj::bounds::DeclUser;
use godot::prelude::*;
use crate::act_react::game_effect::{GameEffectProcessor};


pub type GameEffectInit = fn(Gd<Resource>, &Dictionary, &Dictionary, fn(&dyn GameEffectInitializer, &Dictionary, &Dictionary) -> GameEffectProcessor) -> GameEffectProcessor;
static mut EFFECTS_REGISTRY: Option<HashMap<StringName, GameEffectInit>> = None;


pub fn effects_registry() -> &'static HashMap<StringName, GameEffectInit> {

    unsafe {
        if EFFECTS_REGISTRY.is_none() {
            EFFECTS_REGISTRY = Some(HashMap::new());
        }
        EFFECTS_REGISTRY.as_ref().unwrap()
    }
}


pub fn register_effect_builder<T>(name: StringName)
    where
    T: Inherits<Resource> + GodotClass + Bounds<Declarer = DeclUser> + GameEffectInitializer
{
    unsafe {
        if EFFECTS_REGISTRY.is_none() {
            EFFECTS_REGISTRY = Some(HashMap::new());
        }
        EFFECTS_REGISTRY.as_mut().unwrap().entry(name).or_insert_with(
            || {
                |base, act_context, world_context, closure| {
                    let mut instance = base.cast::<T>();
                    let guard: GdMut<T> = instance.bind_mut();
                    closure(&*guard, act_context, world_context)
                }
            }
        );
    }
}


pub trait GameEffectInitializer: Debug {
    fn build(&self, act_context: &Dictionary, context: &Dictionary) -> GameEffectProcessor;
}