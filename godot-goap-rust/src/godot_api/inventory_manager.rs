use std::cmp::Ordering;
use std::collections::{HashMap};
use godot::classes::Engine;
use godot::prelude::*;
use crate::godot_api::godot_inventory::{InventoryAgent, ItemToSpawn};
use crate::godot_api::item_object::{Item, ItemResource};
use crate::inventory::inventory_entity::{InventoryEntity, InventoryEntityResult};
use crate::inventory::inventory_item::StackResult;
use crate::utils::generate_id::assign_id;

#[derive(Debug)]
pub struct InventoryToCreate {
    id: u32,
    size: Vector2i,
    agent: Gd<InventoryAgent>,
}

impl InventoryToCreate {
    pub fn from_agent(agent: &InventoryAgent) -> Self {
        let id = agent.id;
        let size = agent.size;
        InventoryToCreate {
            id,
            size,
            agent: agent.to_gd(),
        }
    }

    pub fn entity(&self) -> InventoryEntity {
        InventoryEntity::new(self.size.x as usize, self.size.y as usize)
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
        Some(self.cmp(other))
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
    #[init(default = Some(Vec::new()))]
    inventories_to_create: Option<Vec<InventoryToCreate>>,
    base: Base<Node>
}

#[godot_api]
impl INode for InventoryManager {
    fn enter_tree(&mut self) {
        Engine::singleton()
            .register_singleton(Self::singleton_name(), self.base().clone().upcast::<Object>());
    }

    fn exit_tree(&mut self) {
        Engine::singleton().unregister_singleton(Self::singleton_name());
    }

    fn ready(&mut self) {
        self.base_mut().call_deferred("create_inventories".into(), &[]);
    }
}

#[godot_api]
impl InventoryManager {
    #[signal]
    fn post_init();

    #[func]
    fn create_inventories(&mut self) {
        self.initialize_inventories();
        self.base_mut().emit_signal("post_init".into(), &[]);
    }

    #[func]
    pub fn get_items(&self, inventory_id: u32) -> Array<Gd<Item>> {
        let inventory = self.inventories.get(&inventory_id).unwrap();
        let mut items: Array<Gd<Item>> = Array::new();
        for id in inventory.get_items() {
            items.push(self.items.get(&id).unwrap().clone());
        }
        items
    }
}

impl InventoryManager {
    pub fn singleton_name() -> StringName {
        StringName::from("InventoryManager")
    }

    pub fn singleton() -> Gd<Self> {
        Engine::singleton()
            .get_singleton(Self::singleton_name())
            .unwrap()
            .cast::<InventoryManager>()
    }

    pub fn register_inventory(&mut self, to_create: InventoryToCreate) {
        self.inventories_to_create.as_mut().unwrap().push(to_create);
    }

    fn remove_item(&mut self, mut item: Gd<Item>) {
        let item_id = item.bind().id;
        self.items.remove(&item_id);
        if let Some(Some(inventory_id)) = item.bind().inventory.as_ref().map(|i| i.current_inventory_id) {
            self.inventories.get_mut(&inventory_id).unwrap().remove_item(item_id);
        }
        item.emit_signal("item_deleted".into(), &[]);
        item.free();
    }

    pub fn reduce_stack(&mut self, mut item: Gd<Item>, by: u32) {
        // bail if no inventory
        let mut item_bind = item.bind_mut();
        let Some(mut inventory_component) = item_bind.inventory.as_mut() else {return; };
        let result = inventory_component.reduce_stack(by);
        drop(item_bind);
        match result {
            StackResult::Depleted => {self.remove_item(item);}
            StackResult::Updated => {item.emit_signal("updated".into(), &[]);}
            StackResult::NoChange | StackResult::WrongType => unreachable!()
        };
    }

    fn create_items_in_inventory(&mut self, item_to_spawn: Gd<ItemToSpawn>, inventory: Option<&mut InventoryEntity>, inventory_id: u32) {
        // inventory might already be owned or not yet initialized. In latter case we are sending a reference to InventoryEntity.
        let mut inventory = inventory.unwrap_or_else(|| self.inventories.get_mut(&inventory_id).expect("no such inventory!"));

        let bind = item_to_spawn.bind();
        let builder = bind.builder().id(&mut self.current_item_id);

        for mut item in builder {
            item.bind_mut().inventory.as_mut().unwrap().current_inventory_id = Some(inventory_id);
            let id = item.bind().id;

            let item_to_insert = if item_to_spawn.bind().assign_position {
                let pos = item_to_spawn.bind().position;
                let size = inventory.get_size();
                let at_index = pos.y as usize * size.0 + pos.x as usize;
                // panics if user tries to spawn multiple items
                let Ok(item) = inventory.try_insert_item_at(item, at_index) else { panic!("couldn't initialize inventory!") };
                item
            } else {
                // panics if there isn't space for given item in the inventory
                let Ok(item) = inventory.insert_at_first_free_space(item) else { panic!("couldn't initialize inventory!") };
                item
            };
            self.items.insert(id, item_to_insert);
        }
    }

    pub fn create_item(&mut self, item_to_spawn: Gd<ItemToSpawn>, inventory_id: u32) {
        self.create_items_in_inventory(item_to_spawn, None, inventory_id);
    }

    pub fn check_grid_cells(&self, item: Gd<Item>, inventory_id: u32, position_idx: usize) -> InventoryEntityResult {
        let Some(inventory) = self.inventories.get(&inventory_id) else {return InventoryEntityResult::WrongItemType(item);};
        inventory.check_at(item, position_idx)
    }

    pub fn move_item(&mut self, mut item: Gd<Item>, inventory_id: u32, position_idx: usize) -> Result<Gd<Item>, InventoryEntityResult> {
        let Some(inventory) = self.inventories.get_mut(&inventory_id) else {return Err(InventoryEntityResult::WrongItemType(item));};

        if let Some(other_item_id) = inventory.get_item_id_at(position_idx) {
            if other_item_id != item.bind().id {
                // try to stack item
                let mut other_item = self.items.get_mut(&other_item_id).unwrap().bind_mut();
                let result = {

                    let mut item_bind = item.bind_mut();
                    let inventory_component = item_bind.inventory.as_mut().unwrap();
                    let other_inventory_component = other_item.inventory.as_mut().unwrap();
                    inventory_component.stack(other_inventory_component)
                };

                return match result {
                    StackResult::WrongType => { Err(InventoryEntityResult::SpaceTaken(vec![(position_idx, other_item_id)], item)) }
                    StackResult::Updated => {
                        other_item.base_mut().emit_signal("updated".into(), &[]);
                        Ok(item)
                    }
                    StackResult::NoChange => { Ok(item) }
                    // remove item
                    StackResult::Depleted => {
                        other_item.base_mut().emit_signal("updated".into(), &[]);
                        std::mem::drop(other_item);
                        self.remove_item(item);
                        Err(InventoryEntityResult::ItemDepleted)
                    }
                };
            }

            // bail if no change
            return Ok(item);
        }

        let result = { inventory.try_insert_item_at(item, position_idx)};


        match result {
            Ok(mut item) => {
                let previous_inventory = item.bind().inventory.as_ref().unwrap().current_inventory_id;
                let inventory_changed = previous_inventory.map(|p_id| p_id != inventory_id).unwrap_or(false);
                if inventory_changed {
                    if let Some(inv) = self.inventories.get_mut(previous_inventory.as_ref().unwrap()) {
                        inv.remove_item(item.bind().id);
                    }
                    item.emit_signal("inventory_switched".into(), &[inventory_id.to_variant()]);
                }
                item.bind_mut().inventory.as_mut().unwrap().current_inventory_id = Some(inventory_id);
                Ok(item)
            },
            Err(e) => {Err(e)}
        }
    }

    fn initialize_inventories(&mut self) {
        for mut to_create in self.inventories_to_create.take().unwrap().drain(..) {
            let mut inventory = to_create.entity();
            let inventory_id = assign_id(to_create.agent.bind().id, &mut self.current_inventory_id);

            for item_to_spawn in to_create.agent.bind().items_to_spawn.iter_shared() {
                self.create_items_in_inventory(item_to_spawn, Some(&mut inventory), inventory_id);
                // let bind = item_to_spawn.bind();
                // let builder = bind.builder().id(&mut self.current_item_id);
                //
                // for mut item in builder {
                //     item.bind_mut().inventory.as_mut().unwrap().current_inventory_id = Some(inventory_id);
                //     let id = item.bind().id;
                //
                //     let item_to_insert = if item_to_spawn.bind().assign_position {
                //         let pos = item_to_spawn.bind().position;
                //         let size = inventory.get_size();
                //         let at_index = pos.y as usize * size.0 + pos.x as usize;
                //         // panics if user tries to spawn multiple items
                //         let Ok(item) = inventory.try_insert_item_at(item, at_index) else {panic!("couldn't initialize inventory!")};
                //         item
                //     } else {
                //         // panics if there isn't space for given item in the inventory
                //         let Ok(item) = inventory.insert_at_first_free_space(item) else {panic!("couldn't initialize inventory!")};
                //         item
                //     };
                //     self.items.insert(id, item_to_insert);
                // }
            }

            to_create.agent.bind_mut().id = inventory_id;
            self.inventories.insert(inventory_id, inventory);
        }
        self.base_mut().emit_signal("inventories_initialized".into(), &[]);
    }
}
