use godot::prelude::*;
use godot::classes::{AnimationPlayer, Control, IControl, InputEvent};
use std::time::SystemTime;
use crate::godot_api::godot_inventory::InventoryAgent;
use crate::godot_api::inventory_manager::InventoryManager;
use crate::inventory_ui::inventory_ui_grid::InventoryUIGrid;
use crate::inventory_ui::inventory_ui_item::InventoryUIItem;
use crate::inventory_ui::inventory_ui_state_machine::{InventoryUIManagerState, InventoryUIDefaultState};


#[derive(GodotClass)]
#[class(init, base=Control)]
pub struct InventoryUIManager {
    #[export]
    pub inventories: Array<Gd<InventoryUIGrid>>,
    #[export]
    animation_player: Option<Gd<AnimationPlayer>>,
    #[export]
    cooldown_time: f64,
    #[init(default = SystemTime::now())]
    cooldown: SystemTime,
    pub player_inventory_ids: [u32; 3],
    #[init(default = Some(Box::<InventoryUIDefaultState>::default()))]
    state: Option<Box<dyn InventoryUIManagerState>>,
    // current cell size. Cached for displaying new temporary inventories
    current_cell_size: f32,
    base: Base<Control>
}

pub struct InventoryUIManagerView<'view> {
    pub inventories: &'view Array<Gd<InventoryUIGrid>>,
    pub cooldown_time: f64,
    pub cooldown: &'view mut SystemTime,
    pub base: Gd<Control>,
}

impl InventoryUIManager {
    pub fn as_view(&mut self) -> InventoryUIManagerView {
        let base = self.base_mut().clone();
        InventoryUIManagerView {
            inventories: &self.inventories,
            cooldown_time: self.cooldown_time,
            cooldown: &mut self.cooldown,
            base,
        }
    }
}

#[godot_api]
impl IControl for InventoryUIManager {
    fn ready(&mut self) {
        let callable = self.base().callable("on_resized");
        for mut grid_holder in self.inventories.iter_shared() {
            grid_holder.connect("resized".into(), callable.clone());
        }
        InventoryManager::singleton().connect("post_init".into(), self.base().callable("on_inventory_init"));
        self.base_mut().call_deferred("calculate_offset".into(), &[]);
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        if let Some(state) = self.state.take() {
            self.state = Some(state.input(event, self.as_view()));
        }
    }
}

#[godot_api]
impl InventoryUIManager {
    #[signal]
    fn cell_size_calculated(new_cell_size: f32);

    #[func]
    fn on_item_pressed(&mut self, item: Gd<InventoryUIItem>) {
        if let Some(state) = self.state.take() {
            self.state = Some(state.press_event(item, self.as_view()));
        }
    }

    #[func]
    fn on_item_frobbed(&mut self, item: Gd<InventoryUIItem>) {
        godot_print!("frob frob");
        if let Some(state) = self.state.take() {
            self.state = Some(state.frob_event(item, self.as_view()));
        }
    }


    #[func]
    fn on_resized(&mut self) {
        self.base_mut().call_deferred("calculate_offset".into(), &[]);
    }

    #[func]
    fn on_inventory_init(&mut self) {
        let player_inventory_agents = self
            .base()
            .get_tree()
            .expect("failed to fetch the scene tree!")
            .get_nodes_in_group("player_inventory".into());
        for (index, id) in player_inventory_agents
            .iter_shared()
            .map(|v| v.cast::<InventoryAgent>().bind().id)
            .enumerate()
        {
            self.player_inventory_ids[index] = id;
        }
    }

    #[func]
    fn calculate_offset(&mut self) {
        let mut offset: Option<f32> = None;
        for grid_holder in self.inventories.iter_shared() {
            if let Ok(inventory_agent) = grid_holder
                .get("inventory_agent".into())
                .try_to::<Gd<InventoryAgent>>()
            {
                let size = grid_holder.get_size();
                // smallest side of the square
                let grid_min_offset = (size.x / inventory_agent.bind().size.x as f32)
                    .min(size.y / inventory_agent.bind().size.y as f32);

                if let Some(off) = offset {
                    if off > grid_min_offset {
                        offset = Some(grid_min_offset);
                    }
                } else {
                    offset = Some(grid_min_offset);
                }
            }
        }
        if let Some(off) = offset {
            self.current_cell_size = off;
            self.base_mut()
                .emit_signal("cell_size_calculated".into(), &[off.to_variant()]);
        }
    }
}
