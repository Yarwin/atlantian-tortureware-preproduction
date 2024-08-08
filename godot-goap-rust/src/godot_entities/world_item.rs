use godot::classes::IRigidBody3D;
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
}

#[godot_api]
impl IRigidBody3D for WorldItem {
    fn ready(&mut self) {
        if self.item.is_none() {
            self.item = Some(InventoryManager::singleton().bind_mut().create_item(self.item_to_spawn.clone().unwrap()));
        }
    }
}
