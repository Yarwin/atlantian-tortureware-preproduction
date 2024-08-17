use std::fs::create_dir_all;
use godot::classes::{Button, Control, IControl, Label, TextureRect, Tween};
use godot::prelude::*;
use crate::ai::planner::plan;
use crate::equipment::equip_component::equipment_component_registry;
use crate::equipment::spreadgun::{SpreadGunAmmo, SpreadGunItemComponent};
use crate::godot_api::CONNECT_DEFERRED;
use crate::godot_api::gamesys::GameSystem;
use crate::godot_api::godot_inventory::InventoryAgent;
use crate::godot_api::inventory_manager::InventoryManager;
use crate::godot_api::item_object::Item;

#[derive(GodotClass)]
#[class(init, base=Control)]
pub struct GunDisplay {
    #[init(node = "GunName")]
    gun_name_label: OnReady<Gd<Label>>,
    #[init(node = "GunUISprite")]
    gun_sprite: OnReady<Gd<TextureRect>>,
    #[init(node = "Control/HBoxContainer/Control/FireMode")]
    firemode_label: OnReady<Gd<Label>>,
    #[init(node = "Control/HBoxContainer/Control2/ChangeAmmoType")]
    ammo_button: OnReady<Gd<Button>>,
    #[init(node = "Control/HBoxContainer2/Control/AmmoName")]
    ammo_name_label: OnReady<Gd<Label>>,
    #[init(node = "Control/HBoxContainer/Control3/AmmoCount")]
    ammo_count_label: OnReady<Gd<Label>>,
    #[init(node = "Control/HBoxContainer2/Control2/AmmoCountInInventory")]
    ammo_count_in_inventory_label: OnReady<Gd<Label>>,
    #[init(node = "GunStatusLabel")]
    gun_status_label: OnReady<Gd<Label>>,
    available_ammo: Option<Gd<SpreadGunAmmo>>,
    current_ammo: Option<Gd<SpreadGunAmmo>>,
    inventories: Vec<u32>,
    status_tween: Option<Gd<Tween>>,
    /// direct pointer to given item component. Is valid as long as its Item is valid.
    pub eq_component: Option<*mut SpreadGunItemComponent>,
    base: Base<Control>
}


impl GunDisplay {
    fn init_new_ammo(&mut self, ammo: Gd<SpreadGunAmmo>) {
        let ammo_bind = ammo.bind();
        self.ammo_name_label.set_text(ammo_bind.ammo_name.clone());
    }

    fn update_ammo_count_internal_only(&mut self, eq_component: &mut SpreadGunItemComponent) {
        self.ammo_count_label.set_text(format!("{}", eq_component.ammo_count).into());
    }

    fn update_ammo_count(&mut self, eq_component: &mut SpreadGunItemComponent) {
        self.update_ammo_count_internal_only(eq_component);
        if let Some(current_ammo) = self.current_ammo.as_ref() {
            let mut ammo_count: u32 = 0;
            let ammo_bind = current_ammo.bind();
            let Some(accepted_ammo) = ammo_bind.accepted_ammo.clone() else {return;};
            let ammo_bind = accepted_ammo.bind();
            let Some(inventory_item_data) = ammo_bind.inventory.as_ref().map(|i|  i.clone()) else { return; };
            for inv_idx in self.inventories.iter() {
                let items = InventoryManager::singleton().bind().get_items_of_the_same_type(*inv_idx, inventory_item_data.clone());
                for item in items.iter_shared() {
                    let item_bind = item.bind();
                    let inv = item_bind.inventory.as_ref().unwrap();
                    ammo_count += inv.stack;
                }
            }
            self.ammo_count_in_inventory_label.set_text(format!("{}", ammo_count).into())
        }
    }
}


#[godot_api]
impl IControl for GunDisplay {
    fn ready(&mut self) {
        let player_inventories = self.base().get_tree().unwrap().get_nodes_in_group("player_inventory".into());
        let callable = self.base().callable("on_new_item_created");
        for mut inventory_agent in player_inventories.iter_shared().map(|i| i.cast::<InventoryAgent>()) {
            inventory_agent.connect_ex("new_item_created".into(), callable.clone()).flags(CONNECT_DEFERRED).done();
            inventory_agent.connect_ex("stack_updated".into(), callable.clone()).flags(CONNECT_DEFERRED).done();
            self.inventories.push(inventory_agent.bind().id);
        }
        if let Some(mut eq_component) = self.eq_component.map(|c| unsafe { &mut *c }) {
            if let Some(current_ammo) = eq_component.current_ammo.as_ref().map(|a| a.clone()) {
                self.init_new_ammo(current_ammo);
            }
            self.current_ammo = eq_component.current_ammo.clone();
            self.update_ammo_count(eq_component);
            let gun_name = eq_component.data.bind().gun_name.clone();
            self.gun_name_label.set_text(gun_name);
        }
    }
}

#[godot_api]
impl GunDisplay {
    #[signal]
    fn intent_to_reload();

    #[func(gd_self)]
    fn on_new_item_created(mut this: Gd<Self>, item: Gd<Item>) {
        let mut change = false;
        {
            let this_bind = this.bind();
            let Some(current_ammo) = this_bind.current_ammo.as_ref() else {return;};
            let current_ammo_bind = current_ammo.bind();
            let Some(item_resource) = current_ammo_bind.accepted_ammo.as_ref() else { return; };
            let item_resource_bind = item_resource.bind();
            let Some(inventory_item_data) = item_resource_bind.inventory.as_ref() else { return; };
            let item_bind = item.bind();
            let Some(other_inventory_item_data) = item_bind.inventory.as_ref().map(|i| &i.inventory_data) else { return; };
            if inventory_item_data == other_inventory_item_data {
                change = true;
            }
        }
        if change {
            let mut this_bind = this.bind_mut();
            let Some(eq_comp) = this_bind.eq_component.as_mut().map(|c| unsafe { &mut **c }) else {return;};
            this_bind.update_ammo_count(eq_comp);
        }
    }

    #[func(gd_self)]
    fn on_gun_removed(mut this: Gd<Self>) {

    }

    #[func(gd_self)]
    fn on_new_ammo_type_selected(mut this: Gd<Self>) {

    }

    #[func(gd_self)]
    fn on_new_firemode_selected(mut this: Gd<Self>) {

    }

    #[func]
    fn add_dot_to_status(&mut self) {
        let text = self.gun_status_label.get_text();
        self.gun_status_label.set_text(format!("{}.", text).into());
    }

    #[func]
    fn clear_dots_from_status(&mut self) {
        let mut text = self.gun_status_label.get_text().to_string();
        if text.len() < 2 {
            return;
        }
        self.gun_status_label.set_text(GString::from(&text[0.. text.len() - 1]));
    }

    #[func]
    fn on_gun_status_changed(&mut self, new_status: GString) {
        if let Some(mut tween) = self.status_tween.take() {
            tween.kill();
        }
        let is_empty = new_status.is_empty();
        self.gun_status_label.set_text(new_status);

        if is_empty {return;}
        let mut tween = self.base().get_tree().unwrap().create_tween().unwrap();
        let add_dot_to_status = self.base().callable("add_dot_to_status");
        let remove_dots = self.base().callable("clear_dots_from_status");
        tween = tween.bind_node(self.base().clone()).unwrap();
        tween = tween.set_loops().unwrap();
        tween.tween_callback(add_dot_to_status.clone()).unwrap().set_delay(0.2).unwrap();
        tween.tween_callback(add_dot_to_status.clone()).unwrap().set_delay(0.1).unwrap();
        tween.tween_callback(add_dot_to_status).unwrap().set_delay(0.05).unwrap();
        tween.tween_callback(remove_dots.clone()).unwrap().set_delay(0.2).unwrap();
        tween.tween_callback(remove_dots.clone()).unwrap().set_delay(0.1).unwrap();
        tween.tween_callback(remove_dots).unwrap().set_delay(0.005).unwrap();
        self.status_tween = Some(tween);
    }

    #[func]
    fn on_reloaded(&mut self) {
        let Some(eq_comp) = self.eq_component.as_mut().map(|c| unsafe { &mut **c }) else {return;};
        self.update_ammo_count(eq_comp);
    }

    #[func]
    fn on_shoot(&mut self) {
        let Some(eq_comp) = self.eq_component.as_mut().map(|c| unsafe { &mut **c }) else {return;};
        self.update_ammo_count_internal_only(eq_comp);
    }

    #[func]
    fn on_ammo_changed(&mut self, new_ammo: Gd<SpreadGunAmmo>) {

    }
}