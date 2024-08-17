use godot::classes::{Control, Engine};
use godot::obj::{bounds, Bounds, NewAlloc};
use godot::prelude::*;
use crate::godot_api::item_object::Item;


/// A node responsible to manage communications between different game systems.
#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct GameSys {
    base: Base<Node>,
}

#[godot_api]
impl INode for GameSys {
    fn enter_tree(&mut self) {
        Engine::singleton().register_singleton(Self::singleton_name(), self.base().clone().upcast::<Object>());
    }
    fn exit_tree(&mut self) {
        Engine::singleton().unregister_singleton(Self::singleton_name());
    }
}

#[godot_api]
impl GameSys {
    #[signal]
    fn new_item_put_into_slot(slot: u32, item: Gd<Item>);
    #[signal]
    fn item_removed_from_slot(slot: u32, item: Gd<Item>);
    #[signal]
    fn hud_visibility_changed(hidden: bool);
    #[signal]
    fn new_hitscan_collision_registered(place: Vector3, normal: Vector3);
    #[signal]
    fn new_gun_for_ui_display(gun_ui: Gd<Control>);
}

impl GameSys {
    pub fn singleton() -> Gd<Self> {
        Engine::singleton()
            .get_singleton(Self::singleton_name())
            .unwrap()
            .cast::<GameSys>()
    }

    fn singleton_name() -> StringName {
        StringName::from("GameSystems")
    }
}

pub trait GameSystem: GodotClass + Bounds<Declarer = bounds::DeclUser> + NewAlloc + Inherits<Object> {
    const NAME: &'static str;

    fn singleton() -> Gd<Self> {
        Engine::singleton()
            .get_singleton(Self::singleton_name())
            .unwrap()
            .cast::<Self>()
    }
    fn singleton_name() -> StringName {
        StringName::from(Self::NAME)
    }
    fn initialize() -> Gd<Self> {
        let game_system = Self::new_alloc();
        Engine::singleton()
            .register_singleton(Self::singleton_name(), game_system.clone());
        game_system
    }

    fn exit(&mut self) {Engine::singleton().unregister_singleton(Self::singleton_name());}
    #[allow(unused_variables)]
    fn physics_process(&mut self, delta: f64) {}
    #[allow(unused_variables)]
    fn process(&mut self, delta: f64) {}
}
