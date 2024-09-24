use godot::classes::Engine;
use godot::obj::{bounds, Bounds, NewAlloc};
use godot::prelude::*;
use crate::equipment::equip_component::EquipmentComponent;
use crate::godot_api::ai_manager::GodotAIManager;
use crate::godot_api::inventory_manager::InventoryManager;
use crate::godot_api::item_object::Item;
use crate::multi_function_display::mfd_main::DisplayType;


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
    fn new_ui_item_equipped(item: EquipmentComponent, ui_display: DisplayType);
    #[signal]
    fn ui_item_taken_off();
    #[signal]
    fn frob_prompt_updated(description: GString, progress: f64, name: GString);
    #[signal]
    fn frob_progress_updated(progress: f64);
    #[signal]
    fn frob_description_deactivated();
    #[signal]
    fn new_log_message(message: GString);
    #[signal]
    fn new_debug_info(info: GString);

    /// called after level has been loaded.
    /// forces initialization of all Game Systems
    #[func]
    fn on_level_loaded(&self) {
        InventoryManager::singleton().call_deferred("create_inventories".into(), &[]);
        GodotAIManager::singleton().call_deferred("create_thinkers".into(), &[]);
    }
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
