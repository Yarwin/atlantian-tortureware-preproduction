use std::cmp::Ordering;
use godot::obj::Bounds;
use godot::obj::bounds::DeclUser;
use godot::prelude::*;

pub struct ToCreate<T: Inherits<Object> + GodotClass + Bounds<Declarer = DeclUser>> {
    pub(crate) id: u32,
    pub(crate) instance: Gd<T>
}

impl<T: Inherits<Object> + GodotClass + Bounds<Declarer = DeclUser>> PartialEq for ToCreate<T> {
    fn eq(&self, other: &Self) -> bool {
        other.id == self.id
    }
}

impl<T: Inherits<Object> + GodotClass + Bounds<Declarer = DeclUser>> Eq for ToCreate<T> {}

impl<T: Inherits<Object> + GodotClass + Bounds<Declarer = DeclUser>> PartialOrd for ToCreate<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Inherits<Object> + GodotClass + Bounds<Declarer = DeclUser>> Ord for ToCreate<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}


pub fn assign_id(initial_id: u32, current_max: &mut u32) -> u32 {
    if initial_id != 0 {
        if *current_max < initial_id {
            *current_max = initial_id;
        }
        return initial_id
    }
    *current_max += 1;
    *current_max
}