use godot::prelude::*;
use godot::classes::{Texture, Texture2D};
use godot::classes::notify::ObjectNotification;
use crate::inventory::grid::ItemSize;

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct InventoryItemData {
    #[init(default = 1)]
    #[export]
    pub max_stack: u32,
    #[init(default=(Vector2i::new(0, 0)))]
    #[export]
    pub rectangular_grid_size: Vector2i,
    pub item_size: Option<ItemSize>,
    #[init(default=None)]
    #[export]
    pub texture: Option<Gd<Texture>>,
    #[init(default=None)]
    #[export]
    pub texture_folded: Option<Gd<Texture2D>>,
}

#[godot_api]
impl IResource for InventoryItemData {
}

#[godot_api]
impl InventoryItemData {
    #[func]
    fn initialize(&mut self) {
        self.set_size();
    }
}

impl InventoryItemData {
    fn set_size(&mut self) {
        if self.rectangular_grid_size != Vector2i::ZERO {
            self.item_size = Some(ItemSize::new_rectangular(self.rectangular_grid_size.x as usize, self.rectangular_grid_size.y as usize));
            godot_print!("{}", self.item_size.as_ref().unwrap());
        }
    }
}
