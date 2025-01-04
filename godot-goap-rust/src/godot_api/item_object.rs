use crate::equipment::equip_component::ItemEquipmentComponent;
use crate::inventory::inventory_item::InventoryItem;
use crate::inventory::inventory_item_data::InventoryItemData;
use godot::classes::notify::ObjectNotification;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct ItemResource {
    #[export]
    pub name: StringName,
    #[export]
    pub inventory: Option<Gd<InventoryItemData>>,
    #[export]
    pub equipment: Option<Gd<Resource>>,
}

#[derive(GodotClass)]
#[class(init, base=Object)]
pub struct Item {
    pub id: u32,
    pub item_resource: Option<Gd<ItemResource>>,
    // responsible for managing inventory interactions
    pub inventory: Option<InventoryItem>,
    // responsible for managing equipment data (ammo count and whatnot) & creating equipment scenes for the player
    pub equip: Option<Box<dyn ItemEquipmentComponent>>,
    #[base]
    pub(crate) base: Base<Object>,
}

impl Item {
    pub fn get_item_display(&self) -> GString {
        let item_name = self.item_resource.as_ref().unwrap().bind().name.to_string();
        let Some(inventory_component) = self.inventory.as_ref() else {
            return GString::from(item_name);
        };
        if inventory_component.stack > 1 {
            return GString::from(format!("{} {item_name}", inventory_component.stack));
        }
        GString::from(item_name)
    }
}

#[godot_api]
impl Item {
    #[signal]
    fn equipped();
    #[signal]
    fn taken_off();
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
                .emit_signal("deleted", &[base_clone.to_variant()]);
        }
    }
}
