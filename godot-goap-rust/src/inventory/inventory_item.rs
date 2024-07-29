use godot::prelude::*;
use crate::act_react::act_react_resource::ActReactResource;
use crate::inventory::inventory_item_data::InventoryItemData;


pub enum StackResult {
    NoChange,
    Depleted,
    Updated,
}

pub struct InventoryItem {
    pub current_inventory_id: Option<u32>,
    pub inventory_data: Gd<InventoryItemData>,
    pub stack: u32,
    pub location: Vector2i,
    // how item interact with everything else while in inventory
    pub act_react: Option<Gd<ActReactResource>>
}

impl InventoryItem {
    pub fn stack(&mut self, other: &mut InventoryItem) -> StackResult {
        // bail if items have different type
        if other.inventory_data != self.inventory_data {
            return StackResult::NoChange;
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