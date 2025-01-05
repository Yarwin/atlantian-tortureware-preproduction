
pub enum EffectResult {
    /// effect has been executed and command object can be freed
    Free,
    /// effect should be reverted after `n` seconds
    Revert(f64),
    /// Failed to execute given command
    Failed,
}

impl PartialEq for EffectResult {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EffectResult::Free, EffectResult::Free) => true,
            (EffectResult::Revert(_), EffectResult::Revert(_)) => true,
            (EffectResult::Failed, EffectResult::Failed) => true,
            (_, _) => false,
        }
    }
}

impl Eq for EffectResult {}

// pub type GameEffectDispatch =
//     fn(Gd<Object>, fn(&mut dyn GameEffect) -> EffectResult) -> EffectResult;
//
// #[derive(Debug)]
// pub struct GameEffectProcessor {
//     base: Gd<Object>,
//     // TW: programming crimes
//     // stupid but works
//     // we are creating reference to GameObject – "hidden" behind Gd<Object> smart pointer
//     // on "fly" and use closure to run generic GameEffect trait
//     // it is stupid, but massively decreases boilerplate
//     trait_object_dispatch: GameEffectDispatch,
// }
//
// impl GameEffectProcessor {
//     /// removes the command object at the end of the physics frame
//     pub fn free(&mut self) {
//         self.base.call_deferred("free", &[]);
//     }
//
//     pub fn instance_id(&self) -> InstanceId {
//         self.base.instance_id()
//     }
//
//     pub fn new<T>(base: Gd<T>) -> Self
//     where
//         T: Inherits<Object> + GodotClass + Bounds<Declarer = DeclUser> + GameEffect,
//     {
//         Self {
//             base: base.upcast(),
//             trait_object_dispatch: |base, closure| {
//                 let mut effect_obj: Gd<T> = base.cast::<T>();
//                 let mut guard: GdMut<T> = effect_obj.bind_mut();
//                 closure(&mut *guard)
//             },
//         }
//     }
// }
//
// impl GameEffect for GameEffectProcessor {
//     fn execute(&mut self) -> EffectResult {
//         (self.trait_object_dispatch)(self.base.clone(), |effect: &mut dyn GameEffect| {
//             effect.execute()
//         })
//     }
//     fn revert(&mut self) -> EffectResult {
//         (self.trait_object_dispatch)(self.base.clone(), |effect: &mut dyn GameEffect| {
//             effect.revert()
//         })
//     }
// }

// /// a godot wrapper for commands
// #[derive(GodotClass, Debug)]
// #[class(init, base=Object, rename=GameEffect)]
// pub struct GameEffectWrapper {
//     // having Box<dyn T> inside Gd<T> is stupid – it would be better to store given command inside object (to be considered after adding inheritance to godot-rust)
//     pub effect: Option<Box<dyn GameEffect>>,
//     #[base]
//     base: Base<Object>
// }

pub trait GameEffect {
    fn execute(&mut self) -> EffectResult;
    fn revert(&mut self) -> EffectResult {
        EffectResult::Free
    }
}
