use std::time::SystemTime;
use godot::classes::{Control, IControl, InputEvent, InputEventMouseButton, Label, ShaderMaterial, TextureRect, Timer};
use godot::classes::control::MouseFilter;
use godot::global::MouseButton;
use godot::prelude::*;
use crate::act_react::act_react_resource::ActReactResource;
use crate::godot_api::{CONNECT_DEFERRED};
use crate::godot_api::item_object::Item;
use crate::inventory_ui::inventory_ui_controller::InventoryUIManager;
use crate::inventory_ui::inventory_ui_grid::InventoryUIGrid;


/// a struct responsible for displaying items in the inventory/UI
#[derive(GodotClass)]
#[class(init, base = Control)]
pub struct InventoryUIItem {
    /// an item object related to this UI widget
    #[var]
    pub item: Option<Gd<Item>>,
    #[var]
    pub current_inventory_ui: Option<Gd<InventoryUIGrid>>,
    #[var]
    pub inventory_ui_items_manager: Option<Gd<InventoryUIManager>>,
    #[init(node = "TextureRect")]
    texture_rect: OnReady<Gd<TextureRect>>,
    #[init(node = "TextureRect/Label")]
    amount_label: OnReady<Gd<Label>>,
    // #[init(node = "TextureRect/DurabilityLabel")]
    // durability_label: OnReady<Gd<Label>>,
    #[init(node = "TextureRect/SlotLabel")]
    slot_label: OnReady<Gd<Label>>,
    #[init(node = "Timer")]
    hold_item_timer: OnReady<Gd<Timer>>,
    #[export]
    default_shine_color: Color,
    #[export]
    frob_shine_color: Color,
    #[export]
    cooldown: f64,
    #[init(default = SystemTime::now() )]
    last_cooldown: SystemTime,
    pub is_held: bool,
    is_waiting_for_resize: bool,
    base: Base<Control>,
}


#[godot_api]
impl IControl for InventoryUIItem {
    fn gui_input(&mut self, event: Gd<InputEvent>) {
        // bail if no item or if item is being held
        if self.item.is_none() || self.is_held {
            return;
        }
        let Ok(mouse_button_event) = event.try_cast::<InputEventMouseButton>() else {return;};
        // bail if no mouse input event
        if !mouse_button_event.is_pressed() || mouse_button_event.get_button_index() != MouseButton::LEFT {return;}
        // bail if cooldown is still active

        // frob on doubleclick
        if mouse_button_event.is_double_click() {
            self.hold_item_timer.stop();
            let base_variant = self.base().to_variant();
            // avoid re-entrant by using call deferred and postponing signal emission
            self.base_mut().call_deferred("emit_signal".into(), &[StringName::from("item_frobbed").to_variant(), base_variant]);
            self.base().get_viewport().unwrap().set_input_as_handled();
            self.last_cooldown = SystemTime::now();
            return;
        }
        if self.last_cooldown.elapsed().unwrap().as_secs_f64() < self.cooldown {return;}
        self.base().get_viewport().unwrap().set_input_as_handled();
        self.hold_item_timer.start();
    }

    fn ready(&mut self) {
        let mut ui_items_manager = self.inventory_ui_items_manager.clone().unwrap();
        let press_callable = ui_items_manager.callable("on_item_pressed");
        let frob_callable = ui_items_manager.callable("on_item_frobbed");
        self.base_mut().connect("item_pressed".into(), press_callable);
        self.base_mut().connect("item_frobbed".into(), frob_callable);

        let on_frob_started = self.base().callable("on_frob_started");
        ui_items_manager.connect("inventory_frob_started".into(), on_frob_started);
        let on_frob_finished = self.base().callable("on_frob_finished");
        ui_items_manager.connect("inventory_frob_finished".into(), on_frob_finished);
    }

}


#[godot_api]
impl InventoryUIItem {
    #[signal]
    fn item_frobbed(inventory_item: Gd<InventoryUIItem>);
    #[signal]
    fn item_pressed(inventory_item: Gd<InventoryUIItem>);

    #[func]
    fn on_frob_started(&mut self, other_act_react: Gd<ActReactResource>) {
        let Some(item) = self.item.as_ref() else { return; };
        let item_bind = item.bind();
        let Some(inventory_component) = item_bind.inventory.as_ref() else { return; };
        let inventory_data_bind = inventory_component.inventory_data.bind();
        let Some(act_react) = inventory_data_bind.act_react.as_ref() else { return; };
        if *act_react == other_act_react {
            return;
        }
        if act_react.bind().is_reacting(other_act_react) {
            self.texture_rect.get_material().unwrap().cast::<ShaderMaterial>().set_shader_parameter("shine".into(), 0.6.to_variant());
        }
    }

    #[func]
    fn on_frob_finished(&mut self) {
        self.texture_rect.get_material().unwrap().cast::<ShaderMaterial>().set_shader_parameter("shine".into(), 0.0.to_variant());
        self.texture_rect.get_material().unwrap().cast::<ShaderMaterial>().set_shader_parameter("shin_color".into(), self.default_shine_color.to_variant());
    }

    #[func]
    fn held_item(&mut self) {
        self.is_held = !self.is_held;
        self.base_mut().set_z_index(3);
    }

    #[func(gd_self)]
    pub fn add_item(mut this: Gd<Self>, mut item: Gd<Item>) {
        this.bind_mut().item = Some(item.clone());
        let resize_and_put = this.callable("resize_and_put");
        item.connect_ex("moved".into(), resize_and_put.clone()).flags(CONNECT_DEFERRED).done();
        item.connect_ex("updated".into(), resize_and_put).flags(CONNECT_DEFERRED).done();

        let inventory_switched = this.callable("on_item_inventory_switched");
        item.connect_ex("inventory_switched".into(), inventory_switched).flags(CONNECT_DEFERRED).done();
        let on_item_deleted = this.callable("on_item_deleted");
        item.connect_ex("item_deleted".into(), on_item_deleted).flags(CONNECT_DEFERRED).done();

        let texture = item.bind().inventory.as_ref().expect("no inventory data!").inventory_data.bind().texture.clone().unwrap();
        this.bind_mut().texture_rect.set_texture(texture);
    }

    #[func(gd_self)]
    pub fn on_item_deleted(mut this: Gd<Self>) {
        this.queue_free();
    }

    #[func]
    fn process_resize_and_put(&mut self) {
        self.base_mut().set_mouse_filter(MouseFilter::PASS);
        self.base_mut().set_z_index(0);
        self.last_cooldown = SystemTime::now();
        self.is_held = false;
        self.is_waiting_for_resize = false;
        let item = self.item.as_ref().expect("no item?!").bind();
        let inventory_item = item.inventory.as_ref().unwrap();
        let inventory_ui = self.current_inventory_ui.as_ref().expect("no inventory to put item in!");
        let index = inventory_item.location.x + inventory_item.location.y * inventory_ui.bind().inventory_agent.as_ref().unwrap().bind().size.x;
        let grid_cell = inventory_ui.bind().grid.as_ref().unwrap().get_child(index).expect("wrong index").cast::<Control>();
        let size = grid_cell.get_size() * inventory_item.inventory_data.bind().get_rectangular_grid_size().cast_float();
        if inventory_item.inventory_data.bind().max_stack > 1 {
            self.amount_label.set_text(GString::from(inventory_item.stack.to_string()));
        }
        drop(item);
        self.base_mut().set_global_position(grid_cell.get_global_position());
        self.base_mut().set_size(size);
        self.texture_rect.set_size(size);
    }

    #[func]
    pub fn resize_and_put(&mut self) {
        if self.is_waiting_for_resize {
            return;
        }
        self.is_waiting_for_resize = true;
        self.base_mut().call_deferred("process_resize_and_put".into(), &[]);
    }

    #[func(gd_self)]
    fn on_timer_timeout(mut this: Gd<Self>) {
        this.bind_mut().last_cooldown = SystemTime::now();
        let variant = this.clone().to_variant();
        this.emit_signal("item_pressed".into(), &[variant]);
    }

    #[func(gd_self)]
    fn on_item_inventory_switched(mut this: Gd<Self>, new_inventory_id: u32) {
        let grids = this.get_tree().unwrap().get_nodes_in_group("InventoryGridUI".into());
        for item_grid in grids.iter_shared().map(|g| g.cast::<InventoryUIGrid>()) {
            let do_ids_match = item_grid.bind().inventory_agent.as_ref().map(|ia| ia.bind().id == new_inventory_id).unwrap_or(false);
            if do_ids_match {
                this.bind_mut().current_inventory_ui = Some(item_grid.clone());
                InventoryUIGrid::append_item(item_grid, this);
                return;
            }
        }
    }
}

