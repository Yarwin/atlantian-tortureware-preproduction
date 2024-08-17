use std::ops::{Index, IndexMut};
use godot::classes::{InputEvent, InputEventKey, ShapeCast3D};
use godot::prelude::*;
use crate::act_react::act_react_executor::ActReactExecutor;
use crate::act_react::act_react_resource::ActReactResource;
use crate::act_react::game_effect_builder::effects_registry;
use crate::act_react::react_area_3d::ActReactArea3D;
use crate::equipment::equip_component::{Equipment, EquipmentComponent};
use crate::godot_api::{CONNECT_DEFERRED, CONNECT_ONE_SHOT};
use crate::godot_api::gamesys::{GameSys, GameSystem};
use crate::godot_api::godot_inventory::InventoryAgent;
use crate::godot_api::item_object::Item;
use crate::godot_api_reacts::fly::FlyGameEffect;
use crate::godot_entities::rigid_reactive_body3d::WorldObject;
use crate::player_controller::grab_node::GrabNode;

#[derive(Default, Debug)]
pub struct EquippedItems {
    items: [Option<Gd<Item>>; 10]
}


impl Index<u32> for EquippedItems {
    type Output = Option<Gd<Item>>;

    fn index(&self, index: u32) -> &Self::Output {
        &self.items[index as usize]
    }
}

impl IndexMut<u32> for EquippedItems {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self.items[index as usize]
    }
}

impl GodotConvert for EquippedItems {
    type Via = Dictionary;
}

impl ToGodot for EquippedItems {
    fn to_godot(&self) -> Self::Via {
        let mut dict = Dictionary::new();
        for (i, item) in self.items.iter().enumerate() {
            dict.set(i as u32, item.clone())
        }
        dict
    }
}

impl FromGodot for EquippedItems {
    fn try_from_godot(via: Self::Via) -> Result<Self, ConvertError> {
        let mut equipped_items = Self::default();
        /// todo – handle convert errors
        for (i, v) in via.iter_shared().map(|(k, v)| (k.to::<u32>() as usize, v.to::<Option<Gd<Item>>>())) {
            equipped_items.items[i] = v;
        }
        Ok(equipped_items)
    }
}


/// A node responsible for managing the player state and iteracting with the gameworld.
#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct PlayerController {
    #[init(node = "../Head/Camera3D/InterfaceShapeCast")]
    interface_shape_cast: OnReady<Gd<ShapeCast3D>>,
    #[init(node = "../Head/Camera3D")]
    camera: OnReady<Gd<Camera3D>>,
    #[init(node = "../Head/EqHolder")]
    eq_holder: OnReady<Gd<Node3D>>,
    #[init(node = "../Head/Camera3D/GrabNode")]
    pub grab_node: OnReady<Gd<GrabNode>>,
    #[export]
    inventories: Array<Gd<InventoryAgent>>,
    #[export]
    throw_effect: Option<Gd<FlyGameEffect>>,
    inventories_ids: Option<Array<u32>>,
    #[export]
    interface_act_react: Option<Gd<ActReactResource>>,
    equipped_items: EquippedItems,
    active_item: Option<Gd<Item>>,
    equipment_component: Option<EquipmentComponent>,
    base: Base<Node>
}


impl PlayerController {
    fn get_inventories_ids(&mut self) -> Array<u32> {
        return if let Some(inventories_ids) = self.inventories_ids.as_ref() {
            inventories_ids.clone()
        } else {
            let mut new_array: Array<u32> = Array::new();
            for inventory_agent_id in self.inventories.iter_shared().map(|a| a.bind().id) {
                new_array.push(inventory_agent_id);
            }
            self.inventories_ids = Some(new_array.clone());
            new_array
        }
    }
}


#[godot_api]
impl INode for PlayerController {
    fn physics_process(&mut self, _delta: f64) {
        if Input::singleton().is_action_just_pressed("frob".into()) {
            if self.grab_node.bind().attached.is_some() {
                self.grab_node.bind_mut().detach();
                return;
            }
            let actor = self.base().clone();
            let Some(acts) = self.interface_act_react.clone() else {return;};
            self.interface_shape_cast.force_shapecast_update();
            if self.interface_shape_cast.is_colliding() {
                if let Some(Ok(react_area)) = self.interface_shape_cast.get_collider(0).map(|o| o.try_cast::<ActReactArea3D>()) {
                    let inventories_ids = self.get_inventories_ids().clone();
                    let context = dict! {
                        "inventories": inventories_ids,
                        "actor": actor,
                        "reactor": react_area.bind().target.clone(),
                    };
                    ActReactExecutor::singleton().bind_mut().react(acts, react_area.bind().act_react.clone().unwrap(), context);
                }
            }
        }
    }

    fn ready(&mut self) {
        let on_new_item_put_into_slot = self.base().callable("on_new_item_put_into_slot");
        GameSys::singleton().connect("new_item_put_into_slot".into(), on_new_item_put_into_slot);
        let on_item_removed_from_slot = self.base().callable("on_item_removed_from_slot");
        GameSys::singleton().connect("item_removed_from_slot".into(), on_item_removed_from_slot);
    }
    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        // is action just pressed?
        if event.is_pressed() && !event.is_echo() && event.is_action("activate".into()) {
            if self.grab_node.bind().attached.is_some() {
                let reactor = self.grab_node.bind().attached.clone().unwrap();
                self.grab_node.bind_mut().detach();
                if let Some(fly_effect) = self.throw_effect.as_mut() {
                    let context = dict! {
                        "direction": self.camera.get_global_basis().col_c(),
                        "reactor": reactor
                    };
                    let command_init_fn = effects_registry()[&fly_effect.get_class()];
                    let effect = (command_init_fn)(fly_effect.clone().upcast::<Resource>(),
                                                   &Dictionary::new(), &context, |effect, a_context, world_context |
                        {
                            effect.build(a_context, world_context)
                        }
                    );
                    ActReactExecutor::singleton().bind_mut().add_effect(effect.unwrap());
                }
                self.base().get_viewport().unwrap().set_input_as_handled();
                return;
            }
            if let Some(eq_component) = self.equipment_component.as_mut() {
                eq_component.activate();
                self.base().get_viewport().unwrap().set_input_as_handled();
            }
        }
        if !event.is_pressed() && !event.is_echo() && event.is_action("activate".into()) {
            if let Some(eq_component) = self.equipment_component.as_mut() {
                eq_component.deactivate();
                self.base().get_viewport().unwrap().set_input_as_handled();
            }
        }
    }

    fn unhandled_key_input(&mut self, event: Gd<InputEvent>) {
        if !event.is_pressed() || event.is_echo() {
            return;
        }
        if let Ok(e) = event.try_cast::<InputEventKey>() {
            for i in 0..6 {
                let action_name = format!("slot_{}", i+1);
                if e.is_action(StringName::from(action_name)) {
                    let slot_item = self.equipped_items[i].clone();
                    let active_item = self.active_item.take();

                    if let Some(mut new_item) = slot_item {
                        if let Some(mut prev_item) = active_item {
                            if prev_item == new_item {
                                self.active_item = Some(prev_item);
                                return;
                            }
                            let Some(mut eq_component) = self.equipment_component.take() else {panic!("active item without eq component!")};
                            eq_component.take_off();
                            let on_item_taken_off = self.base().callable("on_old_item_taken_off");
                            prev_item.connect_ex("taken_off".into(), on_item_taken_off).flags(CONNECT_ONE_SHOT + CONNECT_DEFERRED).done();
                            self.active_item = Some(new_item);
                            return;
                        }
                        {
                            let mut item_bind = new_item.bind_mut();
                            let Some(eq_component) = item_bind.equip.as_mut() else { return; };
                            let (mut eq_component, ui) = eq_component.initialize_equipment_scene();
                            drop(item_bind);
                            eq_component.initialize(new_item.clone());
                            self.eq_holder.add_child(&eq_component.base);
                            self.equipment_component = Some(eq_component);
                            GameSys::singleton().emit_signal("new_gun_for_ui_display".into(), &[ui.to_variant()]);
                        }
                        self.active_item = Some(new_item);
                        return;
                    }

                    let Some(mut eq_component) = self.equipment_component.take() else {return;};
                    eq_component.take_off();
                    return
                }
            }
        }
    }


}


#[godot_api]
impl PlayerController {
    #[func(gd_self)]
    fn on_new_item_put_into_slot(mut this: Gd<Self>, slot: u32, item: Gd<Item>) {
        let slot_idx = slot - 1;
        let removed_item = {
            let mut removed_item = None;
            if let Some(previous_item) = this.bind_mut().equipped_items[slot_idx].as_ref() {
                if *previous_item != item {
                    removed_item = Some(previous_item.clone())
                }
            }
            let is_slotted_index = this
                .bind_mut()
                .equipped_items
                .items
                .iter()
                .enumerate()
                .filter_map(|(index, slotted_item)| {
                    if slotted_item.as_ref().map(|i| *i ==item).unwrap_or(false) {
                        Some(index)
                    } else { None }
                }).next();
            if let Some(index) = is_slotted_index {
                this.bind_mut().equipped_items.items[index] = None;
            }
            this.bind_mut().equipped_items[slot_idx] = Some(item);
            removed_item
        };
        if let Some(removed_item) = removed_item {
            GameSys::singleton().emit_signal("item_removed_from_slot".into(), &[slot_idx.to_variant(), removed_item.to_variant()]);
        }
    }

    #[func(gd_self)]
    fn on_item_removed_from_slot(mut this: Gd<Self>, slot: u32, item: Gd<Item>) {
        if this.bind_mut().equipped_items[slot].as_ref().map(|i| *i == item).unwrap_or(false) {
            this.bind_mut().equipped_items[slot] = None;
        }
    }

    #[func]
    fn on_old_item_taken_off(&mut self) {
        let Some(mut new_item) = self.active_item.take() else {return;};
        let mut item_bind = new_item.bind_mut();
        let Some(eq_component) = item_bind.equip.as_mut() else { return; };
        let (mut eq_component, ui) = eq_component.initialize_equipment_scene();
        drop(item_bind);
        eq_component.initialize(new_item.clone());
        self.eq_holder.add_child(&eq_component.base);
        self.equipment_component = Some(eq_component);
        GameSys::singleton().emit_signal("new_gun_for_ui_display".into(), &[ui.to_variant()]);
        self.active_item = Some(new_item);
    }
}
