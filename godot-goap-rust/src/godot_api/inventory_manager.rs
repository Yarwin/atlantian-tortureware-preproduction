use std::cmp::Ordering;
use std::collections::HashMap;
use godot::classes::Engine;
use godot::prelude::*;
use crate::godot_api::godot_inventory::InventoryAgent;
use crate::godot_api::item_object::Item;
use crate::inventory::inventory_entity::InventoryEntity;


pub struct InventoryToCreate {
    id: u32,
    size: Vector2i,
    agent: Gd<InventoryAgent>,
}

impl InventoryToCreate {
    pub fn from_agent(agent: Gd<InventoryAgent>) -> Self {
        let id = agent.bind().id;
        let size = agent.bind().size;
        InventoryToCreate {
            id,
            size,
            agent,
        }
    }
}

impl PartialEq for InventoryToCreate {
    fn eq(&self, other: &Self) -> bool {
        other.id == self.id
    }
}

impl Eq for InventoryToCreate {}

impl PartialOrd for InventoryToCreate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for InventoryToCreate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

/// An entity responsible for managing and tracking inventories & items.
///
/// Exported (for now on) both as autoload and engine singleton
/// for easy access both from Gdscript and Gdextension library.
///
/// In the future it might be moved to some kind of GameManager autoload.
#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct InventoryManager {
    /// ids
    #[init(default = 0)]
    current_inventory_id: u32,
    #[init(default = 0)]
    current_item_id: u32,
    inventories: HashMap<u32, InventoryEntity>,
    items: HashMap<u32, Gd<Item>>,
    inventories_to_create: Vec<InventoryToCreate>,
    base: Base<Node>
}

#[godot_api]
impl INode for InventoryManager {
    fn enter_tree(&mut self) {
        Engine::singleton()
            .register_singleton("InventoryManager".into(), self.base().clone().upcast::<Object>());
    }

    fn exit_tree(&mut self) {
        Engine::singleton().unregister_singleton("InventoryManager".into());
    }

    fn ready(&mut self) {
        self.base_mut().call_deferred("create_inventories".into(), &[]);
    }
}

#[godot_api]
impl InventoryManager {
    #[func]
    fn create_inventories(&mut self) {
        self.register_inventories();
    }
}

impl InventoryManager {
    pub fn singleton() -> Gd<Self> {
        Engine::singleton()
            .get_singleton("InventoryManager".into())
            .unwrap()
            .cast::<InventoryManager>()
    }

    pub fn register_inventory(&mut self, to_create: InventoryToCreate) {
        self.inventories_to_create.push(to_create);
    }

    fn register_inventories(&mut self) {
        self.inventories_to_create.sort();
    }
}
