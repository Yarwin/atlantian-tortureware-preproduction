use crate::inventory::inventory_item_data::InventoryItemData;
use godot::prelude::*;

#[derive(Debug, Eq, PartialEq)]
pub enum StackResult {
    WrongType,
    NoChange,
    Depleted,
    Updated,
}

#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub current_inventory_id: Option<u32>,
    pub inventory_data: Gd<InventoryItemData>,
    pub stack: u32,
    pub location: Vector2i,
}

impl InventoryItem {
    pub fn reduce_stack(&mut self, by: u32) -> StackResult {
        if self.stack <= by {
            return StackResult::Depleted;
        }
        self.stack -= by;
        StackResult::Updated
    }

    pub fn stack(&mut self, other: &mut InventoryItem) -> StackResult {
        // bail if items have different type
        if other.inventory_data != self.inventory_data {
            return StackResult::WrongType;
        }
        let max_stack = self.inventory_data.bind().max_stack;
        let other_stack = other.stack;
        // bail if item is already S T A C K E D
        if other_stack >= max_stack {
            return StackResult::NoChange;
        }
        let free_space = max_stack - other_stack;
        // transfer whole stack to different item
        if free_space >= self.stack {
            other.stack += self.stack;
            return StackResult::Depleted;
        }
        other.stack += free_space;
        self.stack -= free_space;
        StackResult::Updated
    }
}

impl From<Gd<InventoryItemData>> for InventoryItem {
    fn from(value: Gd<InventoryItemData>) -> Self {
        InventoryItem {
            current_inventory_id: None,
            inventory_data: value,
            stack: 1,
            location: Default::default(),
        }
    }
}
