use std::cmp::Ordering;
use std::collections::{HashMap};
use godot::classes::Engine;
use godot::prelude::*;
use crate::godot_api::gamesys::GameSystem;
use crate::godot_api::godot_inventory::{InventoryAgent, ItemToSpawn};
use crate::godot_api::item_object::{Item};
use crate::inventory::inventory_entity::{InventoryEntity, InventoryEntityResult};
use crate::inventory::inventory_item::StackResult;
use crate::inventory::inventory_item_data::InventoryItemData;
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
#[class(init, base=Object)]
pub struct InventoryManager {
    /// ids
    #[init(default = 0)]
    current_inventory_id: u32,
    #[init(default = 0)]
    current_item_id: u32,
    inventories: HashMap<u32, InventoryEntity>,
    inventory_agents: HashMap<u32, Gd<InventoryAgent>>,
    items: HashMap<u32, Gd<Item>>,
    #[init(default = Some(Vec::new()))]
    inventories_to_create: Option<Vec<InventoryToCreate>>,
    pub is_initialized: bool,
    base: Base<Object>
}

#[godot_api]
impl InventoryManager {
    #[func]
    pub fn get_inventory_agent(&self, inventory_idx: u32) -> Option<Gd<InventoryAgent>> {
        self.inventory_agents.get(&inventory_idx).cloned()
    }

    #[signal]
    fn post_init();

    #[func(gd_self)]
    fn create_inventories(mut this: Gd<Self>) {
        this.bind_mut().initialize_inventories();
        this.bind_mut().is_initialized = true;
        this.emit_signal("post_init".into(), &[]);
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

    #[func]
    pub fn get_items_of_the_same_type(&self, inventory_id: u32, inventory_data: Gd<InventoryItemData>) -> Array<Gd<Item>> {
        let inventory = self.inventories.get(&inventory_id).unwrap();
        let mut items: Array<Gd<Item>> = Array::new();
        for id in inventory.get_items() {
            let item = self.items.get(&id).unwrap();
            let item_bind = item.bind();
            let Some(other_inventory_data) = item_bind.inventory.as_ref().map(|i| &i.inventory_data) else { panic!("no inventory data") };
            if inventory_data == *other_inventory_data {
                drop(item_bind);
                items.push(item.clone());
            }
        }
        items
    }
}

impl InventoryManager {

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
        let Some(inventory_component) = item_bind.inventory.as_mut() else {return; };
        let result = inventory_component.reduce_stack(by);
        drop(item_bind);
        match result {
            StackResult::Depleted => {self.remove_item(item);}
            StackResult::Updated => {item.emit_signal("updated".into(), &[]);}
            StackResult::NoChange | StackResult::WrongType => unreachable!()
        };
    }

    fn update_item_and_return_result(&mut self, result: Result<Gd<Item>, InventoryEntityResult>, inventory_id: u32) -> Result<Gd<Item>, InventoryEntityResult> {
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
                if previous_inventory.is_none() {
                    item.emit_signal("inventory_switched".into(), &[inventory_id.to_variant()]);
                    if let Some(inventory_agent) = self.inventory_agents.get_mut(&inventory_id) {
                        inventory_agent.emit_signal("new_item_created".into(), &[item.to_variant()]);
                    }
                }
                Ok(item)
            },
            Err(e) => {Err(e)}
        }
    }

    fn create_items_in_inventory(&mut self, item_to_spawn: Gd<ItemToSpawn>, inventory: Option<&mut InventoryEntity>, inventory_id: u32) -> bool {
        // inventory might already be owned or not yet initialized. In latter case we are sending a reference to InventoryEntity.
        let inventory = inventory.unwrap_or_else(|| self.inventories.get_mut(&inventory_id).expect("no such inventory!"));
        let mut inventory_agent = self.inventory_agents.get_mut(&inventory_id);

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
                let Ok(item) = inventory.try_insert_item_at(item, at_index) else { return false };
                item
            } else {
                // panics if there isn't space for given item in the inventory
                let Ok(item) = inventory.insert_at_first_free_space(item) else { return false };
                item
            };
            if let Some(ref mut agent) = inventory_agent {
                agent.emit_signal("new_item_created".into(), &[item_to_insert.to_variant()]);
            }
            self.items.insert(id, item_to_insert);
        }
        true
    }

    pub fn put_item_at_first_free_space(&mut self, item: Gd<Item>, inventory: Option<&mut InventoryEntity>, inventory_id: u32) -> Result<Gd<Item>, InventoryEntityResult> {
        let inventory = inventory.unwrap_or_else(|| self.inventories.get_mut(&inventory_id).expect("no such inventory!"));
        let result = inventory.insert_at_first_free_space(item);
        self.update_item_and_return_result(result, inventory_id)
    }

    pub fn create_item_in_inventory(&mut self, item_to_spawn: Gd<ItemToSpawn>, inventory_id: u32) -> bool {
        self.create_items_in_inventory(item_to_spawn, None, inventory_id)
    }

    pub fn try_stack_item(&mut self, mut item: Gd<Item>, mut other: Gd<Item>) -> StackResult {
        if item == other {
            return StackResult::WrongType;
        }
        let result = {
            let mut item_bind = item.bind_mut();
            let mut other_item_bind = other.bind_mut();
            let Some(inventory_component) = item_bind.inventory.as_mut() else {return StackResult::WrongType};
            let Some(other_inventory_component) = other_item_bind.inventory.as_mut() else {return StackResult::WrongType};
            if inventory_component.inventory_data != other_inventory_component.inventory_data {
                return StackResult::WrongType;
            }
            inventory_component.stack(other_inventory_component)
        };

        match &result {
            StackResult::Depleted => {
                other.emit_signal("updated".into(), &[]);
                self.remove_item(item);
            }
            StackResult::Updated => {
                item.emit_signal("updated".into(), &[]);
                other.emit_signal("updated".into(), &[]);
            }
            StackResult::WrongType => unreachable!(),
            _ => {}
        };
        result
    }

    /// creates & registers given item.
    pub fn create_item(&mut self, item_to_spawn: Gd<ItemToSpawn>) -> Gd<Item> {
        let bind = item_to_spawn.bind();
        let builder = bind.builder().id(&mut self.current_item_id);
        let item = builder.build();
        self.items.insert(item.bind().id, item.clone());
        item
    }

    pub fn check_grid_cells(&self, item: Gd<Item>, inventory_id: u32, position_idx: usize) -> InventoryEntityResult {
        let Some(inventory) = self.inventories.get(&inventory_id) else {return InventoryEntityResult::WrongItemType(item);};
        inventory.check_at(item, position_idx)
    }

    pub fn get_item_at(&self, inventory_id: u32, position_idx: usize) -> Option<Gd<Item>> {
        let inventory = self.inventories.get(&inventory_id)?;
        let item_id = inventory.get_item_id_at(position_idx)?;
        self.items.get(&item_id).cloned()
    }

    // todo – inventory manager shouldn't actually decide if items should be stacked and whatnot – such stuff should be handled in implementation details (UI and whatnot)
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
        }

        let result = { inventory.try_insert_item_at(item, position_idx)};
        self.update_item_and_return_result(result, inventory_id)
    }

    fn initialize_inventories(&mut self) {
        for mut to_create in self.inventories_to_create.take().unwrap().drain(..) {
            let mut inventory = to_create.entity();
            let inventory_id = assign_id(to_create.agent.bind().id, &mut self.current_inventory_id);

            for item_to_spawn in to_create.agent.bind().items_to_spawn.iter_shared() {
                if !self.create_items_in_inventory(item_to_spawn, Some(&mut inventory), inventory_id) {
                    panic!("failed to initialize inventory!");
                }
            }

            to_create.agent.bind_mut().id = inventory_id;
            self.inventories.insert(inventory_id, inventory);
            self.inventory_agents.insert(inventory_id, to_create.agent);
        }
        self.base_mut().emit_signal("inventories_initialized".into(), &[]);
    }
}


impl GameSystem for InventoryManager {
    const NAME: &'static str = "InventoryManager";
    fn singleton_name() -> StringName {
        StringName::from("InventoryManager")
    }

    fn initialize() -> Gd<Self> {
        let mut inventory_manager = Self::new_alloc();
        Engine::singleton()
            .register_singleton(Self::singleton_name(), inventory_manager.clone());
        inventory_manager.call_deferred("create_inventories".into(), &[]);
        inventory_manager
    }

    fn exit(&mut self) {
        Engine::singleton().unregister_singleton(Self::singleton_name());
        for (_id, item) in self.items.drain() {
            item.free();
        }
    }
}