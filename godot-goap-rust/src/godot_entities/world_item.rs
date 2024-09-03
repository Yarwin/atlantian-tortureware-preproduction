use godot::classes::{IRigidBody3D, RigidBody3D};
use godot::prelude::*;
use crate::godot_api::gamesys::GameSystem;
use crate::godot_api::godot_inventory::ItemToSpawn;
use crate::godot_api::inventory_manager::InventoryManager;
use crate::godot_api::item_object::{Item};


/// a representation of given item in 3D space
#[derive(GodotClass, Debug)]
#[class(init, base=RigidBody3D)]
pub struct WorldItem {
    #[export]
    pub item_to_spawn: Option<Gd<ItemToSpawn>>,
    #[var]
    pub item: Option<Gd<Item>>,
    base: Base<RigidBody3D>
}

#[godot_api]
impl IRigidBody3D for WorldItem {
    fn ready(&mut self) {
        if self.item.is_none() {
            self.item = Some(InventoryManager::singleton().bind_mut().create_item(self.item_to_spawn.clone().unwrap()));
        }
        let callable = self.base().callable("on_item_removed");
        self.item.as_mut().unwrap().connect("item_deleted".into(), callable.clone());
        let callable = self.base().callable("on_item_picked_up");
        self.item.as_mut().unwrap().connect("inventory_switched".into(), callable.clone());
        self.item.as_mut().unwrap().connect("item_equipped".into(), callable);
    }
}

#[godot_api]
impl WorldItem {
    #[func]
    fn get_name_display(&self) -> GString {
        let Some(item) = self.item.as_ref() else {return return GString::default()};
        let display_text = item.bind().get_item_display();
        display_text
    }

    #[func]
    fn on_item_picked_up(&mut self, _: u32) {
        self.base_mut().queue_free();
    }
    #[func]
    fn on_item_removed(&mut self) {
        self.base_mut().queue_free();
    }
}