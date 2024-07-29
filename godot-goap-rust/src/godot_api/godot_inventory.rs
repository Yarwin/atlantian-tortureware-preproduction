use godot::prelude::*;
use crate::godot_api::inventory_manager::{InventoryManager, InventoryToCreate};


#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct ItemToSpawn {
    #[export]
    amount: u32
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
    base: Base<Node>
}

impl InventoryAgent {
    fn register_inventory(&mut self) {
        let to_create = InventoryToCreate::from_agent(self.to_gd());
        let mut singleton = InventoryManager::singleton();
        singleton.bind_mut().register_inventory(to_create);
        // singleton.bind_mut().inventories_to_create.push((
        //     self.size.x.try_into().unwrap(),
        //     self.size.y.try_into().unwrap(),
        //     self.id,
        //     self.base().clone().cast::<InventoryAgent>(),
        // ));
    }
}

#[godot_api]
impl INode for InventoryAgent {
    fn ready(&mut self) {
        self.register_inventory();
    }
}
