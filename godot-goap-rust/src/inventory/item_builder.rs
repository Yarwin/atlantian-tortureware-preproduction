use crate::inventory::inventory_item::InventoryItem;
use godot::prelude::*;
use crate::equipment::equip_component::build_item_equipment_component;
use crate::godot_api::godot_inventory::ItemToSpawn;
use crate::godot_api::item_object::{Item, ItemResource};

#[derive(Default)]
pub struct ItemBuilder<'a> {
    pub inventory: Option<InventoryItem>,
    pub equipment: Option<Gd<Resource>>,
    amount: u32,
    pub context: Option<Dictionary>,
    spawn_context: Option<Gd<ItemToSpawn>>,
    current_item_id: Option<&'a mut u32>
}

impl<'a> ItemBuilder<'a> {
    pub fn new() -> Self {
        ItemBuilder::<'_> {
            amount: 1,
            ..Default::default()
        }
    }

    pub fn spawn_context(mut self, spawn_context: Gd<ItemToSpawn>) -> Self {
        self.spawn_context = Some(spawn_context);
        self
    }

    pub fn amount(mut self, amount: u32) -> Self {
        self.amount = amount;
        self
    }

    fn resolve_context(&mut self) {

    }

    pub fn context(mut self, context: Dictionary) -> Self {
        self.context = Some(context);
        self.resolve_context();
        self
    }

    pub fn id(mut self, item_id: &'a mut u32) -> Self {
        self.current_item_id = Some(item_id);
        self
    }

    /// creates singular item
    pub fn build(mut self) -> Gd<Item> {
        if let Some(inventory_type) = self.inventory.as_mut() {
            inventory_type.stack = self.amount;
        }
        **self.current_item_id.as_mut().unwrap() += 1;
        Gd::<Item>::from_init_fn(|base| Item {
            id: *self.current_item_id.unwrap(),
            inventory: self.inventory.take(),
            equip: None,
            base
        })
    }
}

impl Iterator for ItemBuilder<'_> {
    type Item = Gd<Item>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.amount == 0 {
            return None;
        }
        // let item: Gd<Item>;
        let max_stack = self.inventory.as_ref()?.inventory_data.bind().max_stack;
        **self.current_item_id.as_mut()? += 1;
        let equip = self.equipment.as_ref().map(|e| build_item_equipment_component(e.clone()));
        let item = if self.amount <= max_stack {
            self.inventory.as_mut()?.stack = self.amount;
            self.amount = 0;
            Gd::<Item>::from_init_fn(|base| Item {
                id: *self.current_item_id.take().unwrap(),
                inventory: self.inventory.take(),
                equip,
                base
            })
        } else {
            self.inventory.as_mut()?.stack = max_stack;
            self.amount -= max_stack;
            Gd::<Item>::from_init_fn(|base| Item {
                id: **self.current_item_id.as_ref().unwrap(),
                inventory: self.inventory.clone(),
                equip,
                base
            })
        };
        Some(item)
    }
}

impl From<&Gd<ItemResource>> for ItemBuilder<'_> {
    fn from(value: &Gd<ItemResource>) -> Self {
        let mut builder = ItemBuilder::new();
        let blueprint = value.bind();
        if let Some(data) = blueprint.inventory.as_ref() {
            builder.inventory = Some(InventoryItem::from(data.clone()));
        }
        if let Some(data) = blueprint.equipment.as_ref() {
            builder.equipment = Some(data.clone());
        }
        builder
    }
}
