use godot::prelude::*;
use godot::classes::{InputEvent, InputEventMouseButton, InputEventMouseMotion};
use crate::godot_api::inventory_manager::InventoryManager;
use crate::inventory::inventory_entity::InventoryEntityResult;
use crate::inventory_ui::inventory_ui_item::InventoryUIItem;
use crate::inventory_ui::inventory_ui_controller::InventoryUIManagerView;

pub trait InventoryUIManagerState {
    fn input(
        self: Box<Self>,
        event: Gd<InputEvent>,
        inventory_ui_manager: InventoryUIManagerView,
    ) -> Box<dyn InventoryUIManagerState>;
    fn press_event(
        self: Box<Self>,
        presser: Gd<InventoryUIItem>,
        inventory_ui_manager: InventoryUIManagerView,
    ) -> Box<dyn InventoryUIManagerState>;
    fn frob_event(
        self: Box<Self>,
        frobber: Gd<InventoryUIItem>,
        inventory_ui_manager: InventoryUIManagerView,
    ) -> Box<dyn InventoryUIManagerState>;

    fn hide_event(
        self: Box<Self>,
        inventory_ui_manager: InventoryUIManagerView
    ) -> Box<dyn InventoryUIManagerState>;
}

#[derive(Default)]
pub struct InventoryUIDefaultState;

impl InventoryUIManagerState for InventoryUIDefaultState {
    fn input(self: Box<Self>, _event: Gd<InputEvent>, _inventory_ui_manager: InventoryUIManagerView) -> Box<dyn InventoryUIManagerState> {
        self
    }

    fn press_event(self: Box<Self>, mut presser: Gd<InventoryUIItem>, inventory_ui_manager: InventoryUIManagerView) -> Box<dyn InventoryUIManagerState> {
        if inventory_ui_manager.cooldown.elapsed().unwrap().as_secs_f64() < inventory_ui_manager.cooldown_time {
            return self;
        }
        presser.set_z_index(2);
        Box::new(
            InventoryUIMoveItemState {
                item_held: presser
            }
        )
        // return Box::new()
    }

    fn frob_event(self: Box<Self>, frobber: Gd<InventoryUIItem>, inventory_ui_manager: InventoryUIManagerView) -> Box<dyn InventoryUIManagerState> {
        todo!()
    }

    fn hide_event(self: Box<Self>, inventory_ui_manager: InventoryUIManagerView) -> Box<dyn InventoryUIManagerState> {
        self
    }
}

pub struct InventoryUIMoveItemState {
    pub item_held: Gd<InventoryUIItem>,
}


impl InventoryUIMoveItemState {
    fn release_item(&mut self, mouse_position: Vector2, inventory_ui_manager: &mut InventoryUIManagerView) -> Result<Box<dyn InventoryUIManagerState>, ()> {

        let mut ui_item_bind = self.item_held.bind_mut();
        // UGLY: clone Gd<Item> smart pointer to avoid re-entrant (requires bind on ui_item_bind)
        let mut item = ui_item_bind.item.as_mut().unwrap().clone();
        let mut inventory_manager = InventoryManager::singleton();

        for inventory_grid in inventory_ui_manager.inventories.iter_shared() {
            let area_rect = inventory_grid.get_global_rect();
            // bail if item outside given inventory space
            if !area_rect.has_point(mouse_position) {
                continue
            }

            let inventory_id = inventory_grid.bind().inventory_agent.as_ref().unwrap().bind().id;
            let Some(index) = inventory_grid.bind().global_coords_to_index(mouse_position) else {continue};
            let result = inventory_manager.bind_mut().move_item(item, inventory_id, index);
            match result {
                // bail if item no longer exists
                Err(InventoryEntityResult::ItemDepleted) => {
                    std::mem::drop(ui_item_bind);
                    self.unhighlight_grid();
                    return Ok(Box::new(InventoryUIDefaultState));
                }
                _ => {
                    item = result.unwrap_or_else(|e| e.item());
                    break
                }
            }
        }

        std::mem::drop(ui_item_bind);
        // move item to its current position (one before or after movement)
        item.emit_signal("moved".into(), &[]);
        self.unhighlight_grid();
        Ok(Box::new(InventoryUIDefaultState))
    }

    fn highlight_grid(&mut self, event: Gd<InputEventMouseMotion>, inventory_ui_manager: InventoryUIManagerView) {

    }

    fn unhighlight_grid(&mut self) {

    }
}

impl InventoryUIManagerState for InventoryUIMoveItemState {
    fn input(mut self: Box<Self>, event: Gd<InputEvent>, mut inventory_ui_manager: InventoryUIManagerView) -> Box<dyn InventoryUIManagerState> {
        let event_cast = event.try_cast::<InputEventMouseButton>();
        if let Ok(mouse_event) = event_cast {
            if !mouse_event.is_pressed() {return self}
            if let Ok(state) = self.release_item(mouse_event.get_global_position(), &mut inventory_ui_manager) {
                inventory_ui_manager.base.get_viewport().unwrap().set_input_as_handled();
                return state;
            }
            return self;
        }

        let Ok(mouse_motion) = event_cast.err().unwrap().try_cast::<InputEventMouseMotion>() else {return self};
        let item_global_pos = self.item_held.get_global_position();
        self.item_held.set_global_position(item_global_pos.lerp(mouse_motion.get_global_position(), 0.9));
        self.highlight_grid(mouse_motion, inventory_ui_manager);
        self
    }

    fn press_event(mut self: Box<Self>, presser: Gd<InventoryUIItem>, mut inventory_ui_manager: InventoryUIManagerView) -> Box<dyn InventoryUIManagerState> {
        if inventory_ui_manager.cooldown.elapsed().unwrap().as_secs_f64() > inventory_ui_manager.cooldown_time {
            if let Ok(state) = self.release_item(inventory_ui_manager.base.get_global_mouse_position(), &mut inventory_ui_manager) {
                return state;
            }
        }
        self
    }

    fn frob_event(self: Box<Self>, _frobber: Gd<InventoryUIItem>, _inventory_ui_manager: InventoryUIManagerView) -> Box<dyn InventoryUIManagerState> {
        self
    }

    fn hide_event(self: Box<Self>, inventory_ui_manager: InventoryUIManagerView) -> Box<dyn InventoryUIManagerState> {
        todo!()
    }
}