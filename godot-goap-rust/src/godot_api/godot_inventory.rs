use crate::godot_api::gamesys::GameSystem;
use crate::godot_api::inventory_manager::{InventoryManager, InventoryToCreate};
use crate::godot_api::item_object::{Item, ItemResource};
use crate::inventory::item_builder::ItemBuilder;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct ItemToSpawn {
    #[export]
    pub amount: u32,
    #[export]
    pub assign_position: bool,
    #[export]
    pub position: Vector2i,
    #[export]
    pub item_data: Option<Gd<ItemResource>>,
    base: Base<Resource>,
}

impl ItemToSpawn {
    pub fn builder(&self) -> ItemBuilder {
        let item_builder = ItemBuilder::from(self.item_data.as_ref().unwrap())
            .amount(self.amount)
            .spawn_context(self.to_gd());
        item_builder
    }
}

/// Facade to manage inventory connected to this entity
#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct InventoryAgent {
    /// Inventory id - leave the 0 to get new assigned id upon registering said entity, enter any given positive id otherwise
    /// Ids might be shared between multiple InventoryAgents (think about vacuum tubes and whatnot) in case if they refer to the very same inventory
    #[export]
    pub id: u32,
    /// size of generated inventory
    #[export]
    pub size: Vector2i,
    /// items to spawn in given inventory
    #[export]
    pub items_to_spawn: Array<Gd<ItemToSpawn>>,
    base: Base<Node>,
}

impl InventoryAgent {}

#[godot_api]
impl INode for InventoryAgent {
    fn ready(&mut self) {
        self.register_inventory();
    }
}

#[godot_api]
impl InventoryAgent {
    #[signal]
    fn new_item_created(item: Gd<Item>);

    #[signal]
    fn stack_updated(item: Gd<Item>);

    #[func]
    pub fn get_items(&self) -> Array<Gd<Item>> {
        InventoryManager::singleton().bind().get_items(self.id)
    }

    #[func]
    fn register_inventory(&mut self) {
        let to_create = InventoryToCreate::from_agent(self);
        let mut singleton = InventoryManager::singleton();
        singleton.bind_mut().register_inventory(to_create);
    }
}
