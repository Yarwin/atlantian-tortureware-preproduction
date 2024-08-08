use crate::godot_api::gamesys::GameSystem;
use godot::prelude::*;
use godot::classes::{Control, IControl, InputEvent, Tween};
use std::time::SystemTime;
use crate::act_react::act_react_resource::ActReactResource;
use crate::godot_api::CONNECT_ONE_SHOT;
use crate::godot_api::gamesys::GameSys;
use crate::godot_api::godot_inventory::InventoryAgent;
use crate::godot_api::inventory_manager::InventoryManager;
use crate::inventory_ui::inventory_ui_grid::InventoryUIGrid;
use crate::inventory_ui::inventory_ui_item::InventoryUIItem;
use crate::inventory_ui::inventory_ui_state_machine::{InventoryUIManagerState, InventoryUIDefaultState};


#[derive(GodotClass)]
#[class(init, base=Control)]
pub struct InventoryUIManager {
    #[init(node = "../../CanvasLayer/ItemHolder")]
    items_holder: OnReady<Gd<Control>>,
    #[export]
    pub inventories: Array<Gd<InventoryUIGrid>>,
    #[var]
    tween: Option<Gd<Tween>>,
    initial_anchors: Vector4,
    items_holder_initial_anchors: Vector4,
    #[export]
    anchors_hidden: Vector4,
    #[export]
    cooldown_time: f64,
    #[init(default = SystemTime::now())]
    cooldown: SystemTime,
    pub player_inventory_ids: Array<u32>,
    #[init(default = Some(Box::<InventoryUIDefaultState>::default()))]
    state: Option<Box<dyn InventoryUIManagerState>>,
    current_focused_grid: Option<Gd<InventoryUIGrid>>,
    // current cell size. Cached for displaying new temporary inventories
    current_cell_size: f32,
    base: Base<Control>
}

pub struct InventoryUIManagerView<'view> {
    pub tween: &'view mut Option<Gd<Tween>>,
    pub initial_anchors: &'view Vector4,
    pub hidden_anchors: &'view Vector4,
    pub items_holder_initial_anchors: &'view Vector4,
    pub items_holder: &'view OnReady<Gd<Control>>,
    pub inventories: &'view Array<Gd<InventoryUIGrid>>,
    pub player_inventory_ids: &'view Array<u32>,
    pub current_focused_grid: &'view mut Option<Gd<InventoryUIGrid>>,
    pub cooldown_time: f64,
    pub cooldown: &'view mut SystemTime,
    pub base: Gd<Control>,
}

impl InventoryUIManager {
    pub fn as_view(&mut self) -> InventoryUIManagerView {
        let base = self.base_mut().clone();
        InventoryUIManagerView {
            tween: &mut self.tween,
            initial_anchors: &self.initial_anchors,
            hidden_anchors: &self.anchors_hidden,
            items_holder_initial_anchors: &self.items_holder_initial_anchors,
            items_holder: &self.items_holder,
            inventories: &self.inventories,
            player_inventory_ids: &self.player_inventory_ids,
            current_focused_grid: &mut self.current_focused_grid,
            cooldown_time: self.cooldown_time,
            cooldown: &mut self.cooldown,
            base,
        }
    }
}

#[godot_api]
impl IControl for InventoryUIManager {
    fn ready(&mut self) {
        self.initial_anchors = Vector4::new(
            self.base().get_anchor(Side::LEFT),
            self.base().get_anchor(Side::TOP),
            self.base().get_anchor(Side::RIGHT),
            self.base().get_anchor(Side::BOTTOM),
        );
        self.items_holder_initial_anchors = Vector4::new(
            self.items_holder.get_anchor(Side::LEFT),
            self.items_holder.get_anchor(Side::TOP),
            self.items_holder.get_anchor(Side::RIGHT),
            self.items_holder.get_anchor(Side::BOTTOM),
        );
        let callable = self.base().callable("on_resized");
        let mouse_entered_callable = self.base().callable("on_mouse_entered_grid");
        let mouse_exited_callable = self.base().callable("on_mouse_exited_grid");
        for mut grid_holder in self.inventories.iter_shared() {
            grid_holder.connect("resized".into(), callable.clone());
            let callable_bind_args = varray![grid_holder.clone().to_variant()];
            grid_holder.connect("mouse_entered".into(), mouse_entered_callable.bindv(callable_bind_args.clone()));
            grid_holder.connect("mouse_exited".into(), mouse_exited_callable.bindv(callable_bind_args));
        }
        if GameSys::singleton().bind().is_initialized {
            self.inventory_initialization();
        } else {
            GameSys::singleton().connect_ex("initialization_completed".into(), self.base().callable("on_inventory_manager_created")).flags(CONNECT_ONE_SHOT).done();
        }

        self.base_mut().call_deferred("calculate_offset".into(), &[]);
        let on_hud_visibility_changed = self.base().callable("on_hud_visibility_changed");
        GameSys::singleton().connect("hud_visibility_changed".into(), on_hud_visibility_changed);
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
    fn inventory_frob_started(frob_act_react: Gd<ActReactResource>);
    #[signal]
    fn inventory_frob_finished();

    #[signal]
    fn cell_size_calculated(new_cell_size: f32);

    #[func]
    fn inventory_initialization(&mut self) {
        // bail if inventory is initialized and controller can move on with initialization
        if InventoryManager::singleton().bind().is_initialized {
            self.on_inventory_init();
            return;
        }
        InventoryManager::singleton().connect_ex("post_init".into(), self.base().callable("on_inventory_init")).flags(CONNECT_ONE_SHOT).done();
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

    #[func]
    fn on_mouse_entered_grid(&mut self, grid: Gd<InventoryUIGrid>) {
        if self.current_focused_grid.as_ref().map(|g| *g == grid).unwrap_or(false) {
            return;
        }
        self.current_focused_grid = Some(grid);
    }

    #[func]
    fn on_mouse_exited_grid(&mut self, _grid: Gd<InventoryUIGrid>) {
        if let Some(grid) = self.current_focused_grid.as_ref() {
            if !grid.get_global_rect().has_point(grid.get_global_mouse_position()) {
                self.current_focused_grid = None;
            }
        }
    }

    #[func]
    fn on_item_pressed(&mut self, item: Gd<InventoryUIItem>) {
        if let Some(state) = self.state.take() {
            self.state = Some(state.press_event(item, self.as_view()));
        }
    }

    #[func]
    fn on_item_frobbed(&mut self, item: Gd<InventoryUIItem>) {
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
        for id in player_inventory_agents
            .iter_shared()
            .map(|v| v.cast::<InventoryAgent>().bind().id)
        {
            self.player_inventory_ids.push(id);
        }
    }

    #[func]
    fn on_hud_visibility_changed(&mut self, is_hidden: bool) {
        if let Some(state) = self.state.take() {
            self.state = Some(state.hide_event(self.as_view(), is_hidden));
        }
    }
}
