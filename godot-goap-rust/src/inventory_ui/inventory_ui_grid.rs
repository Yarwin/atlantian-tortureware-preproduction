use crate::godot_api::gamesys::GameSystem;
use crate::godot_api::godot_inventory::InventoryAgent;
use crate::godot_api::inventory_manager::InventoryManager;
use crate::godot_api::item_object::Item;
use crate::godot_api::{CONNECT_DEFERRED, CONNECT_ONE_SHOT};
use crate::inventory_ui::inventory_ui_controller::InventoryUIManager;
use crate::inventory_ui::inventory_ui_item::InventoryUIItem;
use godot::classes::{Control, GridContainer, IControl, MarginContainer};
use godot::prelude::*;

/// responsible for displaying grid on a screen
#[derive(GodotClass)]
#[class(init, base=Control)]
pub struct InventoryUIGrid {
    #[export]
    pub inventory_ui_items_manager: Option<Gd<InventoryUIManager>>,
    #[export]
    margin_container: Option<Gd<MarginContainer>>,
    #[export]
    pub grid: Option<Gd<GridContainer>>,
    #[export]
    inventory_group: StringName,
    #[export]
    grid_cell_scene: Option<Gd<PackedScene>>,
    #[export]
    item_scene: Option<Gd<PackedScene>>,
    #[export]
    item_holder: Option<Gd<Control>>,
    #[var]
    pub inventory_agent: Option<Gd<InventoryAgent>>,
    offset: f32,
    currently_highlighted_cells: Vec<usize>,
    current_danger_cells: Vec<(usize, u32)>,
    base: Base<Control>,
}

impl InventoryUIGrid {
    fn stop_highlighting_given_cells<'a>(
        cells: impl Iterator<Item = &'a usize>,
        grid: &mut Gd<GridContainer>,
    ) {
        for cell in cells {
            let mut child = grid.get_child(*cell as i32).unwrap().cast::<Control>();
            child.call("unhighlight", &[]);
        }
    }

    pub fn highlight_cells_red(&mut self, cells: Vec<(usize, u32)>) {
        if self.current_danger_cells == cells {
            return;
        }
        let Some(grid) = self.grid.as_mut() else {
            return;
        };

        let iterator = self
            .current_danger_cells
            .iter()
            .filter_map(|(c, id)| (!cells.contains(&(*c, *id))).then_some(c))
            .chain(self.currently_highlighted_cells.iter());
        Self::stop_highlighting_given_cells(iterator, grid);

        for (cell, _id) in cells.iter() {
            let mut child = grid.get_child(*cell as i32).unwrap().cast::<Control>();
            child.call("highlight_red", &[]);
        }
        self.current_danger_cells = cells;
        self.currently_highlighted_cells.clear();
    }

    pub fn highlight_cells(&mut self, cells: Vec<usize>) {
        if self.currently_highlighted_cells == cells {
            return;
        }
        let Some(grid) = self.grid.as_mut() else {
            return;
        };

        let iterator = self
            .currently_highlighted_cells
            .iter()
            .filter(|c| !cells.contains(c))
            .chain(self.current_danger_cells.iter().map(|(c, _id)| c));
        Self::stop_highlighting_given_cells(iterator, grid);

        for cell in cells.iter() {
            let mut child = grid.get_child(*cell as i32).unwrap().cast::<Control>();
            child.call("highlight", &[]);
        }
        self.currently_highlighted_cells = cells;
        self.current_danger_cells.clear();
    }

    pub fn stop_highlighting_all(&mut self) {
        let Some(grid) = self.grid.as_mut() else {
            return;
        };
        for cell in self
            .currently_highlighted_cells
            .drain(..)
            .chain(self.current_danger_cells.drain(..).map(|(c, _id)| c))
        {
            let mut child = grid.get_child(cell as i32).unwrap().cast::<Control>();
            child.call("unhighlight", &[]);
        }
    }

    /// performs binary search to find child idx that has given coords.
    /// todo – this is stupid. Since we know that all the grid elements are rectangles of the same size, we can calculate index directly
    pub fn global_coords_to_index(&self, coords: Vector2) -> Option<usize> {
        let grid = self.grid.as_ref().unwrap();
        let mut min = 0;
        let mut max = grid.get_child_count() - 1;

        loop {
            if min > max {
                return None;
            }
            let midpoint = (min + max) / 2;
            let child = grid
                .get_child(midpoint)
                .expect("logic error, no child with index {midpoint}")
                .cast::<Control>();
            let child_rect = child.get_global_rect();
            if child_rect.has_point(coords) {
                return Some(midpoint as usize);
            }
            let row_equality = coords.y >= child_rect.position.y
                && coords.y <= child_rect.position.y + child_rect.size.y;
            if row_equality {
                if coords.x > child_rect.position.x + child_rect.size.x {
                    min = midpoint + 1;
                    continue;
                }
                max = midpoint - 1;
                continue;
            }
            let midpoint_row_greater = coords.y > child_rect.position.y + child_rect.size.y;
            if midpoint_row_greater {
                min = midpoint + 1;
                continue;
            } else {
                max = midpoint - 1;
            }
        }
    }

    pub fn initialize_grid(&mut self) {
        let Some(grid) = self.grid.as_mut() else {
            return;
        };
        let Some(inventory_agent) = self.inventory_agent.as_ref() else {
            return;
        };
        let Some(grid_cell_scene) = self.grid_cell_scene.as_ref() else {
            return;
        };
        grid.set_columns(inventory_agent.bind().size.x);

        for _i in 0..inventory_agent.bind().size.x * inventory_agent.bind().size.y {
            let grid_cell = grid_cell_scene.instantiate().unwrap();
            grid.add_child(&grid_cell);
        }
    }

    /// returns True if agent has been set&initialized, false otherwise
    pub fn init_inventory_agent(&mut self) -> bool {
        if self.inventory_agent.is_some() {
            return true;
        }
        // bail if no inventory agent & group to get agent from
        if self.inventory_group.is_empty() {
            return false;
        }
        let Some(potential_agent) = self
            .base()
            .get_tree()
            .unwrap()
            .get_first_node_in_group(&self.inventory_group)
            .map(|n| n.cast::<InventoryAgent>())
        else {
            return false;
        };
        self.inventory_agent = Some(potential_agent);
        true
    }
}

#[godot_api]
impl IControl for InventoryUIGrid {
    fn ready(&mut self) {
        self.initialize_inventory();
    }
}

#[godot_api]
impl InventoryUIGrid {
    #[signal]
    fn offset_set();

    #[func]
    fn initialize_inventory(&mut self) {
        if !self.init_inventory_agent() {
            return;
        }
        let on_new_item_created = self.base().callable("on_new_item_created");
        self.inventory_agent
            .as_mut()
            .unwrap()
            .connect_ex("new_item_created", &on_new_item_created)
            .flags(CONNECT_DEFERRED)
            .done();
        let callable = self.base().callable("on_mouse_exited");
        self.base_mut().connect("mouse_exited", &callable);
        let on_cell_size_calculated = self.base_mut().callable("on_cell_size_calculated");
        let on_init = self.base_mut().callable("on_init");
        let Some(inventory_ui_controller) = self.inventory_ui_items_manager.as_mut() else {
            return;
        };
        inventory_ui_controller.connect("cell_size_calculated", &on_cell_size_calculated);
        InventoryManager::singleton()
            .connect_ex("post_init", &on_init)
            .flags(CONNECT_ONE_SHOT)
            .done();
    }

    #[func(gd_self)]
    fn on_new_item_created(this: Gd<Self>, mut item: Gd<Item>) {
        Self::init_item(this, item.clone());
        item.emit_signal("moved", &[]);
    }

    #[func]
    fn on_mouse_exited(&mut self) {
        // handle the cases in which mouse hovers over the control from another layer
        // but still haven't leaved the control's area
        if !self
            .base()
            .get_global_rect()
            .has_point(self.base().get_global_mouse_position())
        {
            self.stop_highlighting_all();
        }
    }

    #[func(gd_self)]
    fn on_init(mut this: Gd<Self>) {
        this.bind_mut().initialize_grid();
        Self::initialize_items(this);
    }

    /// creates&initializes new InventoryUIItem.
    #[func(gd_self)]
    fn init_item(mut this: Gd<Self>, item: Gd<Item>) {
        let mut inventory_item = this
            .bind()
            .item_scene
            .as_ref()
            .unwrap()
            .instantiate()
            .unwrap()
            .cast::<InventoryUIItem>();
        inventory_item
            .bind_mut()
            .inventory_ui_items_manager
            .clone_from(&this.bind().inventory_ui_items_manager);
        this.bind_mut()
            .item_holder
            .as_mut()
            .unwrap()
            .add_child(&inventory_item);
        InventoryUIItem::add_item(inventory_item.clone(), item);
        InventoryUIGrid::append_item(this, inventory_item);
    }

    #[func(gd_self)]
    fn initialize_items(this: Gd<Self>) {
        let items = this
            .bind()
            .inventory_agent
            .as_ref()
            .expect("no items")
            .bind()
            .get_items();
        for item in items.iter_shared() {
            Self::init_item(this.clone(), item);
        }
    }

    #[func]
    fn on_cell_size_calculated(&mut self, new_offset: f32) {
        self.offset = new_offset;
        let Some(inventory_agent) = self.inventory_agent.as_ref() else {
            return;
        };
        let size = self.base().get_size();
        let Some(margin_container) = self.margin_container.as_mut() else {
            return;
        };
        let vertical_offset = (self.offset * inventory_agent.bind().size.y as f32) / 2.0;
        let horizontal_offset = (self.offset * inventory_agent.bind().size.x as f32) / 2.0;
        let offset_top_diff = vertical_offset - (size.y / 2.0);

        // push to top
        // in case if we would like to keep it centered: margin_container.set_offset(Side::TOP, (-vertical_offset).clamp(-size.y / 2.0, 0.0));
        margin_container.set_offset(Side::TOP, -size.y / 2.0);
        // extend bottom by difference from top
        margin_container.set_offset(
            Side::BOTTOM,
            (vertical_offset + offset_top_diff).clamp(0.0, size.y / 2.0),
        );
        margin_container.set_offset(Side::LEFT, (-horizontal_offset).clamp(-size.x, 0.0));
        margin_container.set_offset(Side::RIGHT, horizontal_offset.clamp(0.0, size.x));
        self.base_mut().emit_signal("offset_set", &[]);
    }

    #[func(gd_self)]
    pub fn append_item(mut this: Gd<Self>, mut item: Gd<InventoryUIItem>) {
        let on_resized = item.callable("resize_and_put");
        this.connect_ex("offset_set", &on_resized)
            .flags(CONNECT_DEFERRED)
            .done();
        let on_inventory_switched = this
            .callable("on_item_removed_from_inventory")
            .bindv(&varray![item.to_variant()]);
        item.bind_mut()
            .item
            .as_mut()
            .unwrap()
            .connect_ex("inventory_switched", &on_inventory_switched)
            .flags(CONNECT_ONE_SHOT)
            .done();
        item.bind_mut().current_inventory_ui = Some(this);
    }

    #[func(gd_self)]
    fn on_item_removed_from_inventory(
        mut this: Gd<Self>,
        _new_inventory_id: Variant,
        item: Gd<InventoryUIItem>,
    ) {
        let on_resized = item.callable("resize_and_put");
        this.disconnect("offset_set", &on_resized);
        // let on_inventory_switched = this.callable(StringName::from("on_item_removed_from_inventory")).bindv(array![item.to_variant()]);
        // item.bind_mut().item.as_mut().unwrap().disconnect("inventory_switched", &on_inventory_switched);
    }
}
