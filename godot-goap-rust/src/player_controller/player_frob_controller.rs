use std::ops::{Index, IndexMut};
use std::time::SystemTime;
use godot::classes::{CollisionObject3D, InputEvent, InputEventKey, RigidBody3D, ShapeCast3D};
use godot::prelude::*;
use crate::act_react::act_react_resource::ActReactResource;
use crate::equipment::equip_component::EquipmentComponent;
use crate::godot_api::CONNECT_DEFERRED;
use crate::godot_api::gamesys::{GameSys, GameSystem};
use crate::godot_api::godot_inventory::InventoryAgent;
use crate::godot_api::item_object::Item;
use crate::godot_api_reacts::fly::FlyGameEffect;
use crate::player_controller::grab_node::GrabNode;
use crate::player_controller::player_frob_state_machine::{DefaultState, PlayerEvent, PlayerState};

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
    pub(crate) interface_shape_cast: OnReady<Gd<ShapeCast3D>>,
    #[init(node = "../Head/Camera3D")]
    pub(crate) camera: OnReady<Gd<Camera3D>>,
    #[init(node = "../Head/EqHolder")]
    pub(crate) eq_holder: OnReady<Gd<Node3D>>,
    #[init(node = "../Head/Camera3D/GrabNode")]
    pub grab_node: OnReady<Gd<GrabNode>>,
    #[export]
    pub(crate) inventories: Array<Gd<InventoryAgent>>,
    #[export]
    pub(crate) throw_effect: Option<Gd<FlyGameEffect>>,
    pub(crate) inventories_ids: Option<Array<u32>>,
    #[export]
    pub(crate) interface_act_react: Option<Gd<ActReactResource>>,
    pub(crate) equipped_items: EquippedItems,
    pub(crate) active_item: Option<Gd<Item>>,
    pub(crate) equipment_component: Option<EquipmentComponent>,
    #[init(default = SystemTime::now() )]
    pub(crate) last_shapecast_update: SystemTime,
    #[init(default = 0.25 )]
    #[export]
    shapecast_update_time: f64,
    #[init(default = 0.5 )]
    #[export]
    pub(crate) default_to_frob_time: f64,
    state: Option<Box<dyn PlayerState>>,
    pub is_frobbing: bool,
    base: Base<Node>
}


impl PlayerController {
    pub(crate) fn get_inventories_ids(&mut self) -> Array<u32> {
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
        if self.is_frobbing {
            let state = self.state.take().unwrap_or(DefaultState::new_boxed());
            self.state = Some(state.handle_event(self, PlayerEvent::FrobEvent));
        }

        if Input::singleton().is_action_just_pressed("frob".into()) {
            let state = self.state.take().unwrap_or(DefaultState::new_boxed());
            self.state = Some(state.handle_event(self, PlayerEvent::FrobEvent));
        } else if Input::singleton().is_action_just_released("frob".into()) {
            let state = self.state.take().unwrap_or(DefaultState::new_boxed());
            self.state = Some(state.handle_event(self, PlayerEvent::FrobStopEvent))
        }
        if self.last_shapecast_update.elapsed().unwrap().as_secs_f64() > self.shapecast_update_time {
            self.last_shapecast_update = SystemTime::now();
            let state = self.state.take().unwrap_or(DefaultState::new_boxed());
            self.state = Some(state.handle_event(self, PlayerEvent::ShapeCastUpdateEvent));
        }
    }

    fn ready(&mut self) {
        let on_new_item_put_into_slot = self.base().callable("on_new_item_put_into_slot");
        GameSys::singleton().connect("new_item_put_into_slot".into(), on_new_item_put_into_slot);
        let on_item_removed_from_slot = self.base().callable("on_item_removed_from_slot");
        GameSys::singleton().connect("item_removed_from_slot".into(), on_item_removed_from_slot);
        let on_item_grabbed = self.base().callable("on_item_grabbed");
        self.grab_node.connect("object_grabbed".into(), on_item_grabbed);
        let on_grabbed_released = self.base().callable("on_grabbed_released");
        self.grab_node.connect_ex("object_released".into(), on_grabbed_released).flags(CONNECT_DEFERRED).done();
        if let Some(parent) = self.base().get_parent().map(|p| p.cast::<CollisionObject3D>()) {
            self.interface_shape_cast.add_exception(parent);
        }
        // handle load/save
        if self.active_item.is_none() {
            self.state = Some(DefaultState::new_boxed())
        }
    }

    fn unhandled_input(&mut self, event: Gd<InputEvent>) {
        // is action just pressed?
        if event.is_pressed() && !event.is_echo() && event.is_action("activate".into()) {
            let state = self.state.take().unwrap_or(DefaultState::new_boxed());
            self.state = Some(state.handle_event(self, PlayerEvent::ActivateEvent));
        } else if !event.is_pressed() && !event.is_echo() && event.is_action("activate".into()) {
            let state = self.state.take().unwrap_or(DefaultState::new_boxed());
            self.state = Some(state.handle_event(self, PlayerEvent::DeactivateEvent));
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
                    let state = self.state.take().unwrap_or(DefaultState::new_boxed());
                    self.state = Some(state.handle_event(self, PlayerEvent::SlotSelected(i)));
                }
            }
        }
    }
}


#[godot_api]
impl PlayerController {
    #[func]
    fn on_shapecasted_freed(&mut self) {
        let state = self.state.take().unwrap_or(DefaultState::new_boxed());
        self.state = Some(state.handle_event(self, PlayerEvent::ShapeCastObjectFreed));
    }

    #[func]
    fn on_item_grabbed(&mut self, object: Gd<RigidBody3D>) {
        let state = self.state.take().unwrap_or(DefaultState::new_boxed());
        self.state = Some(state.handle_event(self, PlayerEvent::Grabbed(object)));
    }

    #[func]
    fn on_grabbed_freed(&mut self) {
        let state = self.state.take().unwrap_or(DefaultState::new_boxed());
        self.state = Some(state.handle_event(self, PlayerEvent::ColliderFreed));
    }

    #[func]
    fn on_grabbed_released(&mut self) {
        let state = self.state.take().unwrap_or(DefaultState::new_boxed());
        self.state = Some(state.handle_event(self, PlayerEvent::GrabbedReleased));
    }

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
        let state = self.state.take().unwrap_or(DefaultState::new_boxed());
        state.handle_event(self, PlayerEvent::OldItemTakenOff);
    }
}
