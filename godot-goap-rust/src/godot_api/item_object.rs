use godot::classes::notify::ObjectNotification;
use godot::prelude::*;
use crate::inventory::inventory_item::InventoryItem;
use crate::inventory::inventory_item_data::InventoryItemData;

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct ItemResource {
    #[export]
    pub name: StringName,
    #[export]
    pub inventory: Option<Gd<InventoryItemData>>
}

#[derive(GodotClass)]
#[class(init, base=Object)]
pub struct Item {
    pub id: u32,
    pub inventory: Option<InventoryItem>,
    #[base]
    pub(crate) base: Base<Object>
}


#[godot_api]
impl Item {
    #[signal]
    /// emitted when item changes inventories
    fn inventory_switched(new_inventory_id: u32);
    #[signal]
    /// emitted when item is equipped
    fn item_equipped(slot: u32);
    #[signal]
    /// emitted when item has been updated
    fn updated();
    #[signal]
    /// emitted when item stopped moving by UI
    fn moved();
    #[signal]
    /// emitted when item is being deleted (for any reason)
    fn item_deleted();
}


#[godot_api]
impl IObject for Item {
    fn on_notification(&mut self, what: ObjectNotification) {
        if what == ObjectNotification::PREDELETE {
            let base_clone = self.base().clone();
            self.base_mut()
                .emit_signal("deleted".into(), &[base_clone.to_variant()]);
        }
    }
}
