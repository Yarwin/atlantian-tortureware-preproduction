use std::fmt::Debug;
use godot::obj::Bounds;
use godot::obj::bounds::DeclUser;
use godot::prelude::*;


pub type GameEffectDispatch = fn(Gd<Object>, fn(&mut dyn GameEffect));

#[derive(Debug)]
pub struct GameEffectProcessor {
    base: Gd<Object>,
    // TW: programming crimes
    // stupid but works
    // we are creating reference to GameObject – "hidden" behind Gd<Object> smart pointer
    // on "fly" and use closure to run generic GameEffect trait
    // it is stupid, but massively decreases boilerplate
    trait_object_dispatch: GameEffectDispatch
}

impl GameEffectProcessor {

    pub fn new<T>(base: Gd<T>) -> Self
    where T: Inherits<Object> + GodotClass + Bounds<Declarer = DeclUser> + GameEffect
    {
        Self {
            base: base.upcast(),
            trait_object_dispatch: |base, closure| {
                let mut effect_obj: Gd<T> = base.cast::<T>();
                let mut guard: GdMut<T> = effect_obj.bind_mut();
                closure(&mut *guard);
            },
        }
    }
}

impl GameEffect for GameEffectProcessor {
    fn execute(&mut self) {
        (self.trait_object_dispatch)(self.base.clone(), |effect: &mut dyn GameEffect| {effect.execute()});
        // right now we are just freeing given object, but we might want  to store it in the future – for example to revert the command (undo/time travel/whatever)
        self.base.call_deferred("free".into(), &[]);
    }

}



// /// a godot wrapper for commands
// #[derive(GodotClass, Debug)]
// #[class(init, base=Object, rename=GameEffect)]
// pub struct GameEffectWrapper {
//     // having Box<dyn T> inside Gd<T> is stupid – it would be better to store given command inside object (to be considered after adding inheritance to godot-rust)
//     pub effect: Option<Box<dyn GameEffect>>,
//     #[base]
//     base: Base<Object>
// }


pub trait GameEffect: Debug {
    fn execute(&mut self);
}
