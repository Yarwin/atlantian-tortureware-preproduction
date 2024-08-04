use godot::prelude::*;
use godot::classes::{Texture2D, Resource};
use crate::act_react::act_react_resource::ActReactResource;
use crate::inventory::grid::ItemSize;

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct InventoryItemData {
    #[init(default = 1)]
    #[export]
    pub max_stack: u32,
    /// rectangular item size.
    /// Leave 0,0 if you want to create item with uneven size.
    // todo – figure out how to input ItemSize for non-rectangular items in a painless way
    #[init(default=(Vector2i::new(0, 0)))]
    #[export]
    pub rectangular_grid_size: Vector2i,
    /// real item size
    item_size: Option<ItemSize>,
    #[init(default=None)]
    #[export]
    pub texture: Option<Gd<Texture2D>>,
    #[init(default=None)]
    #[export]
    pub texture_folded: Option<Gd<Texture2D>>,
    /// act/react for inventory interactions
    #[export]
    pub act_react: Option<Gd<ActReactResource>>,
    #[base]
    base: Base<Resource>
}

#[godot_api]
impl InventoryItemData {
}

impl InventoryItemData {
    /// Get reference to item size.
    /// Panics if ItemSize isn't set.
    pub fn get_size_force(&self) -> &ItemSize {
        self.item_size.as_ref().unwrap()
    }

    pub fn get_size(&mut self) -> &ItemSize {
        if self.item_size.is_none() {
            if self.rectangular_grid_size != Vector2i::ZERO {
                self.item_size = Some(ItemSize::new_rectangular(self.rectangular_grid_size.x as usize, self.rectangular_grid_size.y as usize));
            } else {
                panic!("no size for Inventory Item!")
            }
        }
        self.item_size.as_ref().unwrap()
    }
}
