
// todo – it can be actually pushed to Godot Singleton, similarly to godot's ClassDB. No need to dabble with unsafe when you can outsource risk and blame someone else for any and all of your mistakes

// pub type GameEffectInit = fn(
//     Gd<Resource>,
//     &Dictionary,
//     &Dictionary,
//     fn(&dyn GameEffectInitializer, &Dictionary, &Dictionary) -> Option<GameEffectProcessor>,
// ) -> Option<GameEffectProcessor>;
// static mut EFFECTS_REGISTRY: Option<HashMap<GString, GameEffectInit>> = None;
//
// pub fn effects_registry() -> &'static HashMap<GString, GameEffectInit> {
//     unsafe {
//         if EFFECTS_REGISTRY.is_none() {
//             EFFECTS_REGISTRY = Some(HashMap::new());
//         }
//         EFFECTS_REGISTRY.as_ref().unwrap()
//     }
// }
//
// pub fn register_effect_builder<T>(name: GString)
// where
//     T: Inherits<Resource> + GodotClass + Bounds<Declarer = DeclUser> + GameEffectInitializer,
// {
//     unsafe {
//         if EFFECTS_REGISTRY.is_none() {
//             EFFECTS_REGISTRY = Some(HashMap::new());
//         }
//         EFFECTS_REGISTRY
//             .as_mut()
//             .unwrap()
//             .entry(name)
//             .or_insert_with(|| {
//                 |base, act_context, world_context, closure| {
//                     let mut instance = base.cast::<T>();
//                     let guard: GdMut<T> = instance.bind_mut();
//                     closure(&*guard, act_context, world_context)
//                 }
//             });
//     }
// }
//
// pub trait GameEffectInitializer: Debug {
//     fn build(&self, act_context: &Dictionary, context: &Dictionary) -> Option<GameEffectProcessor>;
// }
