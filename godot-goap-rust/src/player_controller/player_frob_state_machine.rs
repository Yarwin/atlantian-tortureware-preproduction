#![allow(clippy::boxed_local)]
use crate::act_react::act_react_executor::ActReactExecutor;
use crate::act_react::game_effect_builder::effects_registry;
use crate::act_react::react_area_3d::ActReactArea3D;
use crate::equipment::equip_component::Equipment;
use crate::godot_api::gamesys::{GameSys, GameSystem};
use crate::godot_api::item_object::Item;
use crate::godot_api::{CONNECT_DEFERRED, CONNECT_ONE_SHOT};
use crate::player_controller::player_frob_controller::PlayerController;
use godot::classes::RigidBody3D;
use godot::prelude::*;
use std::fmt::Debug;
use std::time::SystemTime;

#[derive(Debug, PartialEq, Eq)]
pub enum PlayerEvent {
    /// called on Activate action (LMB)
    ActivateEvent,
    /// called when Activate action is being released
    DeactivateEvent,
    /// called on Frob input action
    FrobEvent,
    /// called when Frob action has been released
    FrobStopEvent,
    /// called when grabbed/attached item has been freed
    ColliderFreed,
    /// called when item has been dropped from inventory
    ItemDropped,
    /// called when some object has been grabbed
    Grabbed(Gd<RigidBody3D>),
    GrabbedReleased,
    SlotSelected(u32),
    OldItemTakenOff,
    ShapeCastUpdateEvent,
    ShapeCastObjectFreed,
}

#[derive(Debug, Default)]
pub struct FrobInfo {
    frobbed: Option<Gd<ActReactArea3D>>,
    frobbed_since: Option<SystemTime>,
}

fn update_shapecast(
    player_frob_controller: &mut PlayerController,
    frob_data: &mut Option<FrobInfo>,
) {
    player_frob_controller
        .interface_shape_cast
        .force_shapecast_update();
    let mut frob_info = frob_data.take().unwrap_or_default();

    if player_frob_controller.interface_shape_cast.is_colliding() {
        let possible_collider = player_frob_controller
            .interface_shape_cast
            .get_collider(0)
            .map(|o| o.try_cast::<ActReactArea3D>());
        let Some(Ok(mut react_area)) = possible_collider else {
            let mut frob_option = Some(frob_info);
            stop_frobbing_and_disconnect(player_frob_controller, &mut frob_option);
            *frob_data = frob_option;
            return;
        };
        let is_already_displayed = frob_info
            .frobbed
            .as_mut()
            .map(|i| *i == react_area)
            .unwrap_or(false);
        if is_already_displayed {
            *frob_data = Some(frob_info);
            return;
        }

        let progress = frob_info
            .frobbed_since
            .as_ref()
            .map(|time| {
                1.0_f64.min(
                    time.elapsed().expect("no elapsed time?!").as_secs_f64()
                        / player_frob_controller.default_to_frob_time,
                )
            })
            .unwrap_or(0.0);
        let area_bind = react_area.bind();
        let message = area_bind
            .act_react
            .as_ref()
            .expect("no act react for given area?!")
            .bind()
            .get_playerfrob_display();
        drop(area_bind);
        let on_shapecasted_freed = player_frob_controller
            .base()
            .callable("on_shapecasted_freed");
        if let Some(mut frobbed) = frob_info.frobbed.take() {
            frobbed.disconnect("tree_exiting", &on_shapecasted_freed.clone());
        }
        react_area
            .connect_ex("tree_exiting", &on_shapecasted_freed)
            .flags(CONNECT_ONE_SHOT)
            .done();
        if !message.is_empty() {
            let name: GString = react_area.bind_mut().get_name();
            GameSys::singleton().emit_signal(
                "frob_prompt_updated",
                &[
                    message.to_variant(),
                    progress.to_variant(),
                    name.to_variant(),
                ],
            );
        }
        // cache colliding react area even if it has no interactions
        frob_info.frobbed = Some(react_area);
    } else if frob_info.frobbed.is_some() {
        let mut frob_option = Some(frob_info);
        stop_frobbing_and_disconnect(player_frob_controller, &mut frob_option);
        frob_info = frob_option.take().unwrap();
    }

    *frob_data = Some(frob_info);
}

fn disconnect_frobbed(
    player_frob_controller: &mut PlayerController,
    frobbed: Option<Gd<ActReactArea3D>>,
) {
    if let Some(mut frobbed) = frobbed {
        let on_shapecasted_freed = player_frob_controller
            .base()
            .callable("on_shapecasted_freed");
        frobbed.disconnect("tree_exiting", &on_shapecasted_freed);
    }
}

fn stop_frobbing_and_disconnect(
    player_frob_controller: &mut PlayerController,
    frob_info: &mut Option<FrobInfo>,
) {
    if let Some(frob_info) = frob_info.as_mut() {
        frob_info.frobbed_since = None;
        disconnect_frobbed(player_frob_controller, frob_info.frobbed.take());
    }
    player_frob_controller.is_frobbing = false;
    GameSys::singleton().emit_signal("frob_description_deactivated", &[]);
}

fn stop_frobbing(player_frob_controller: &mut PlayerController, frob_info: &mut Option<FrobInfo>) {
    stop_frobbing_and_disconnect(player_frob_controller, frob_info);
    GameSys::singleton().emit_signal("frob_description_deactivated", &[]);
}

fn frob(player_frob_controller: &mut PlayerController, frob_info: &mut Option<FrobInfo>) {
    let Some(frob_data) = frob_info.as_mut() else {
        return;
    };
    player_frob_controller.is_frobbing = true;

    if frob_data.frobbed.is_none() {
        frob_data.frobbed_since = None;
        return;
    }

    let frob_time = frob_data.frobbed_since.take().unwrap_or(SystemTime::now());
    let elapsed = frob_time
        .elapsed()
        .expect("no time elapsed?!")
        .as_secs_f64();
    frob_data.frobbed_since = Some(frob_time);

    if elapsed < player_frob_controller.default_to_frob_time {
        let progress = 1.0_f64.min(elapsed / player_frob_controller.default_to_frob_time);
        GameSys::singleton().emit_signal("frob_progress_updated", &[progress.to_variant()]);
        return;
    }
    let Some(react_area) = frob_data.frobbed.as_ref() else {
        return;
    };
    let Some(acts) = player_frob_controller.interface_act_react.clone() else {
        return;
    };
    let actor = player_frob_controller.base().clone();
    let inventories_ids = player_frob_controller.get_inventories_ids().clone();
    let reactor = { react_area.bind().get_reactor() };
    let context = dict! {
        "inventories": inventories_ids,
        "actor": actor,
        "reactor": reactor,
    };
    ActReactExecutor::singleton().bind_mut().react(
        acts,
        react_area
            .bind()
            .act_react
            .clone()
            .expect("no act react for react area!?"),
        context,
    );
    stop_frobbing_and_disconnect(player_frob_controller, frob_info);
    player_frob_controller.is_frobbing = false;
}

fn equip_item(
    player_frob_controller: &mut PlayerController,
    mut new_item: Gd<Item>,
    frob_info: Option<FrobInfo>,
) -> Box<dyn PlayerState> {
    {
        let mut item_bind = new_item.bind_mut();
        let Some(eq_component) = item_bind.equip.as_mut() else {
            panic!("no eq component for given item!");
        };
        let (mut eq_component, ui) = eq_component.initialize_equipment_scene();
        drop(item_bind);
        eq_component.initialize(new_item.clone());
        player_frob_controller
            .eq_holder
            .add_child(&eq_component.base);
        GameSys::singleton().emit_signal(
            "new_ui_item_equipped",
            &[eq_component.clone().to_variant(), ui.to_variant()],
        );
        player_frob_controller.equipment_component = Some(eq_component);
    }
    player_frob_controller.active_item = Some(new_item);
    WeaponState::new_boxed_with_frob(frob_info)
}

fn handle_slot_change(
    player_frob_controller: &mut PlayerController,
    slot: u32,
    frob_info: Option<FrobInfo>,
) -> Result<Box<dyn PlayerState>, Option<FrobInfo>> {
    let slot_item = player_frob_controller.equipped_items[slot].clone();
    let active_item = player_frob_controller.active_item.take();

    if let Some(new_item) = slot_item {
        if let Some(mut prev_item) = active_item {
            if prev_item == new_item {
                player_frob_controller.active_item = Some(prev_item);
                return Err(frob_info);
            }
            let Some(mut eq_component) = player_frob_controller.equipment_component.take() else {
                panic!("active item without eq component!")
            };
            eq_component.take_off();
            let on_item_taken_off = player_frob_controller
                .base()
                .callable("on_old_item_taken_off");
            prev_item
                .connect_ex("taken_off", &on_item_taken_off)
                .flags(CONNECT_ONE_SHOT + CONNECT_DEFERRED)
                .done();
            player_frob_controller.active_item = Some(new_item);
            // no change yet
            return Err(frob_info);
        }
        return Ok(equip_item(player_frob_controller, new_item, frob_info));
    }

    // return to default state
    if let Some(mut eq_component) = player_frob_controller.equipment_component.take() {
        eq_component.take_off();
    }
    Ok(DefaultState::new_boxed_with_frob(frob_info))
}

pub trait PlayerState: Debug {
    fn handle_event(
        self: Box<Self>,
        player_frob_controller: &mut PlayerController,
        event: PlayerEvent,
    ) -> Box<dyn PlayerState>;
}

/// default state.
#[derive(Debug)]
pub(crate) struct DefaultState {
    frob_info: Option<FrobInfo>,
}

impl DefaultState {
    pub fn new_boxed() -> Box<Self> {
        Box::new(Self { frob_info: None })
    }

    pub fn new_boxed_with_frob(frob_info: Option<FrobInfo>) -> Box<Self> {
        Box::new(Self { frob_info })
    }
}

impl PlayerState for DefaultState {
    fn handle_event(
        mut self: Box<Self>,
        player_frob_controller: &mut PlayerController,
        event: PlayerEvent,
    ) -> Box<dyn PlayerState> {
        match event {
            PlayerEvent::ActivateEvent
            | PlayerEvent::ColliderFreed
            | PlayerEvent::GrabbedReleased
            | PlayerEvent::DeactivateEvent
            | PlayerEvent::ItemDropped => {}
            PlayerEvent::FrobEvent => frob(player_frob_controller, &mut self.frob_info),
            PlayerEvent::FrobStopEvent => {
                stop_frobbing(player_frob_controller, &mut self.frob_info)
            }
            PlayerEvent::Grabbed(object) => {
                if let Some(frob_info) = self.frob_info.as_mut() {
                    disconnect_frobbed(player_frob_controller, frob_info.frobbed.take());
                    frob_info.frobbed_since = None;
                }
                return GrabState::enter(object, player_frob_controller);
            }
            PlayerEvent::SlotSelected(slot) => {
                match handle_slot_change(player_frob_controller, slot, self.frob_info.take()) {
                    Ok(state) => return state,
                    Err(frob_info) => self.frob_info = frob_info,
                }
            }
            PlayerEvent::OldItemTakenOff => {
                if player_frob_controller.active_item.is_none() {
                    return self;
                }
                let new_item = player_frob_controller.active_item.take().unwrap();
                return equip_item(player_frob_controller, new_item, self.frob_info.take());
            }
            PlayerEvent::ShapeCastUpdateEvent => {
                update_shapecast(player_frob_controller, &mut self.frob_info);
            }
            PlayerEvent::ShapeCastObjectFreed => {
                let Some(frob_data) = self.frob_info.as_mut() else {
                    return self;
                };
                frob_data.frobbed = None;
                frob_data.frobbed_since = None;
                GameSys::singleton().emit_signal("frob_description_deactivated", &[]);
            }
        };
        self
    }
}

#[derive(Debug)]
pub(crate) struct GrabState {
    grabbed_ref: Gd<RigidBody3D>,
}

impl GrabState {
    fn enter(
        mut item_grabbed: Gd<RigidBody3D>,
        player_controller: &mut PlayerController,
    ) -> Box<Self> {
        let on_grabbed_freed = player_controller.base().callable("on_grabbed_freed");
        item_grabbed
            .connect_ex("tree_exiting", &on_grabbed_freed)
            .flags(CONNECT_ONE_SHOT)
            .done();
        if let Some(equipment) = player_controller.equipment_component.as_mut() {
            equipment.point_down()
        }
        Box::new(GrabState {
            grabbed_ref: item_grabbed,
        })
    }

    fn disconnect(&mut self, player_controller: &mut PlayerController) {
        let on_grabbed_freed = player_controller.base().callable("on_grabbed_freed");
        self.grabbed_ref
            .disconnect("tree_exiting", &on_grabbed_freed);
    }

    fn exit(self: Box<Self>, player_controller: &mut PlayerController) -> Box<dyn PlayerState> {
        if let Some(equipment) = player_controller.equipment_component.as_mut() {
            equipment.point_up();
            return WeaponState::new_boxed();
        }
        // return weapon state if any weapon is active, or default state
        DefaultState::new_boxed()
    }
}

impl PlayerState for GrabState {
    fn handle_event(
        mut self: Box<Self>,
        player_frob_controller: &mut PlayerController,
        event: PlayerEvent,
    ) -> Box<dyn PlayerState> {
        match event {
            PlayerEvent::ActivateEvent => {
                player_frob_controller.grab_node.bind_mut().detach();
                let reactor = self.grabbed_ref.clone();
                let Some(throw_effect) = player_frob_controller.throw_effect.as_mut() else {
                    return self;
                };
                let context = dict! {
                    "direction": player_frob_controller.camera.get_global_basis().col_c(),
                    "reactor": reactor
                };
                let command_init_fn = effects_registry()[&throw_effect.get_class()];
                let effect = (command_init_fn)(
                    throw_effect.clone().upcast::<Resource>(),
                    &Dictionary::new(),
                    &context,
                    |effect, a_context, world_context| effect.build(a_context, world_context),
                );
                ActReactExecutor::singleton()
                    .bind_mut()
                    .add_effect(effect.unwrap());
            }
            PlayerEvent::FrobEvent => {
                player_frob_controller.grab_node.bind_mut().detach();
            }
            PlayerEvent::GrabbedReleased => {
                self.disconnect(player_frob_controller);
                return self.exit(player_frob_controller);
            }
            PlayerEvent::ColliderFreed => {
                return self.exit(player_frob_controller);
            }
            PlayerEvent::SlotSelected(_slot) => {}
            PlayerEvent::ShapeCastUpdateEvent
            | PlayerEvent::FrobStopEvent
            | PlayerEvent::DeactivateEvent
            | PlayerEvent::ItemDropped
            | PlayerEvent::Grabbed(_) => {}
            _ => {}
        }
        self
    }
}

#[derive(Debug)]
pub(crate) struct WeaponState {
    frob_info: Option<FrobInfo>,
}

impl WeaponState {
    fn new_boxed() -> Box<Self> {
        Box::new(Self { frob_info: None })
    }

    fn new_boxed_with_frob(frob_info: Option<FrobInfo>) -> Box<Self> {
        Box::new(Self { frob_info })
    }
}

impl PlayerState for WeaponState {
    fn handle_event(
        mut self: Box<Self>,
        player_frob_controller: &mut PlayerController,
        event: PlayerEvent,
    ) -> Box<dyn PlayerState> {
        match event {
            PlayerEvent::ColliderFreed
            | PlayerEvent::GrabbedReleased
            | PlayerEvent::ItemDropped => {}
            PlayerEvent::OldItemTakenOff => {
                if player_frob_controller.active_item.is_none() {
                    return self;
                }
                let new_item = player_frob_controller.active_item.take().unwrap();
                return equip_item(player_frob_controller, new_item, self.frob_info.take());
            }
            PlayerEvent::ShapeCastUpdateEvent => {
                update_shapecast(player_frob_controller, &mut self.frob_info)
            }
            PlayerEvent::ShapeCastObjectFreed => {
                let Some(frob_data) = self.frob_info.as_mut() else {
                    return self;
                };
                frob_data.frobbed = None;
                frob_data.frobbed_since = None;
                GameSys::singleton().emit_signal("frob_description_deactivated", &[]);
            }
            PlayerEvent::ActivateEvent => {
                if let Some(eq_component) = player_frob_controller.equipment_component.as_mut() {
                    eq_component.activate();
                    player_frob_controller
                        .base()
                        .get_viewport()
                        .unwrap()
                        .set_input_as_handled();
                }
            }
            PlayerEvent::DeactivateEvent => {
                if let Some(eq_component) = player_frob_controller.equipment_component.as_mut() {
                    eq_component.deactivate();
                    player_frob_controller
                        .base()
                        .get_viewport()
                        .unwrap()
                        .set_input_as_handled();
                }
            }
            PlayerEvent::FrobEvent => frob(player_frob_controller, &mut self.frob_info),
            PlayerEvent::FrobStopEvent => {
                stop_frobbing(player_frob_controller, &mut self.frob_info)
            }
            PlayerEvent::Grabbed(object) => {
                if let Some(frob_info) = self.frob_info.as_mut() {
                    disconnect_frobbed(player_frob_controller, frob_info.frobbed.take());
                    frob_info.frobbed_since = None;
                }
                return GrabState::enter(object, player_frob_controller);
            }
            PlayerEvent::SlotSelected(slot) => {
                match handle_slot_change(player_frob_controller, slot, self.frob_info.take()) {
                    Ok(state) => return state,
                    Err(frob_info) => self.frob_info = frob_info,
                }
            }
        }
        self
    }
}
