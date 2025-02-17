use crate::act_react::act_react_executor::ActReactExecutor;
use crate::act_react::act_react_resource::ActReactResource;
use crate::godot_api::gamesys::{GameSys, GameSystem};
use crate::godot_api::inventory_manager::InventoryManager;
use crate::inventory::inventory_entity::InventoryEntityResult;
use crate::inventory_ui::inventory_ui_controller::InventoryUIManagerView;
use crate::inventory_ui::inventory_ui_item::InventoryUIItem;
use godot::classes::control::MouseFilter;
use godot::classes::tween::{EaseType, TransitionType};
use godot::classes::{
    InputEvent, InputEventKey, InputEventMouseButton, InputEventMouseMotion, Tween,
};
use godot::prelude::*;

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
        inventory_ui_manager: InventoryUIManagerView,
        is_hidden: bool,
    ) -> Box<dyn InventoryUIManagerState>;
}

#[derive(Default)]
pub struct InventoryUIDefaultState;

impl InventoryUIManagerState for InventoryUIDefaultState {
    fn input(
        self: Box<Self>,
        event: Gd<InputEvent>,
        inventory_ui_manager: InventoryUIManagerView,
    ) -> Box<dyn InventoryUIManagerState> {
        // put given item in a slot
        if let Ok(e) = event.try_cast::<InputEventKey>() {
            let mouse_position = inventory_ui_manager.base.get_global_mouse_position();
            if !e.is_pressed()
                || e.is_echo()
                || !inventory_ui_manager
                    .base
                    .get_global_rect()
                    .has_point(mouse_position)
            {
                return self;
            }

            for i in 1..6 {
                let action_name = format!("slot_{i}");
                if e.is_action(&action_name) {
                    inventory_ui_manager
                        .base
                        .get_viewport()
                        .unwrap()
                        .set_input_as_handled();
                    let Some(inventory_grid) = inventory_ui_manager.current_focused_grid.as_mut()
                    else {
                        return self;
                    };
                    let inventory_id = inventory_grid
                        .bind()
                        .inventory_agent
                        .as_ref()
                        .unwrap()
                        .bind()
                        .id;
                    let Some(index) = inventory_grid.bind().global_coords_to_index(mouse_position)
                    else {
                        return self;
                    };
                    let Some(item) = InventoryManager::singleton()
                        .bind()
                        .get_item_at(inventory_id, index)
                    else {
                        return self;
                    };
                    // bail if item can't be equipped
                    if item.bind().equip.is_none() {
                        return self;
                    }
                    GameSys::singleton().emit_signal(
                        "new_item_put_into_slot",
                        &[i.to_variant(), item.to_variant()],
                    );
                    return self;
                }
            }
        }
        self
    }

    fn press_event(
        self: Box<Self>,
        mut presser: Gd<InventoryUIItem>,
        inventory_ui_manager: InventoryUIManagerView,
    ) -> Box<dyn InventoryUIManagerState> {
        if inventory_ui_manager
            .cooldown
            .elapsed()
            .unwrap()
            .as_secs_f64()
            < inventory_ui_manager.cooldown_time
        {
            return self;
        }
        presser.set_mouse_filter(MouseFilter::IGNORE);
        presser.set_z_index(2);
        Box::new(InventoryUIMoveItemState { item_held: presser })
    }

    fn frob_event(
        self: Box<Self>,
        frobber: Gd<InventoryUIItem>,
        inventory_ui_manager: InventoryUIManagerView,
    ) -> Box<dyn InventoryUIManagerState> {
        // todo – try https://rust-lang.github.io/rfcs/2497-if-let-chains.html
        // if let Some(item) = frobber.bind().item.as_ref()
        //     && let Some(inventory_component) = item.bind().inventory.as_ref()
        // {
        //
        // }
        // bail if item is not fit to be a frobber
        let frobber_bind = frobber.bind();
        let Some(item) = frobber_bind.item.as_ref() else {
            return self;
        };
        let item_bind = item.bind();
        let Some(inventory_component) = item_bind.inventory.as_ref() else {
            return self;
        };
        let inventory_data_resource_bind = inventory_component.inventory_data.bind();
        let Some(act_react) = inventory_data_resource_bind.act_react.clone() else {
            return self;
        };
        // bail if can't frob
        if act_react.bind().emits.is_empty() {
            return self;
        }

        drop(inventory_data_resource_bind);
        drop(item_bind);
        drop(frobber_bind);

        let frob_state = Box::new(InventoryFrobState {
            frobber,
            frob_act_react: act_react.clone(),
        });
        frob_state.enter(inventory_ui_manager);
        frob_state
    }

    fn hide_event(
        self: Box<Self>,
        inventory_ui_manager: InventoryUIManagerView,
        is_hidden: bool,
    ) -> Box<dyn InventoryUIManagerState> {
        let new_state = Box::new(InventoryHiddenState::default());
        new_state.hide_event(inventory_ui_manager, is_hidden)
    }
}

pub struct InventoryUIMoveItemState {
    item_held: Gd<InventoryUIItem>,
}

impl InventoryUIMoveItemState {
    fn release_item(
        &mut self,
        mouse_position: Vector2,
        inventory_ui_manager: &mut InventoryUIManagerView,
    ) -> Result<Box<dyn InventoryUIManagerState>, ()> {
        let ui_item_bind = self.item_held.bind_mut();
        let mut item = ui_item_bind.item.as_ref().unwrap().clone();
        let mut inventory_manager = InventoryManager::singleton();
        let Some(inventory_grid) = inventory_ui_manager.current_focused_grid.as_mut() else {
            return Err(());
        };
        let inventory_id = inventory_grid
            .bind()
            .inventory_agent
            .as_ref()
            .unwrap()
            .bind()
            .id;
        let Some(index) = inventory_grid.bind().global_coords_to_index(mouse_position) else {
            return Err(());
        };

        let result = inventory_manager
            .bind_mut()
            .move_item(item, inventory_id, index);
        match result {
            // bail if item no longer exists
            Err(InventoryEntityResult::ItemDepleted) => {
                drop(ui_item_bind);
                self.stop_highlighting_all(inventory_ui_manager);
                return Ok(Box::new(InventoryUIDefaultState));
            }
            _ => {
                item = result.unwrap_or_else(|e| e.item());
            }
        }

        drop(ui_item_bind);
        // move item to its current position (one before or after movement)
        item.emit_signal("moved", &[]);
        self.stop_highlighting_all(inventory_ui_manager);
        Ok(Box::new(InventoryUIDefaultState))
    }

    fn highlight_grid(
        &mut self,
        mouse_position: Vector2,
        inventory_ui_manager: InventoryUIManagerView,
    ) {
        let Some(inventory_grid) = inventory_ui_manager.current_focused_grid.as_mut() else {
            return;
        };
        let inventory_id = inventory_grid
            .bind()
            .inventory_agent
            .as_ref()
            .unwrap()
            .bind()
            .id;
        let Some(index) = inventory_grid.bind().global_coords_to_index(mouse_position) else {
            return;
        };
        let item_held_bind = self.item_held.bind();
        let Some(item) = item_held_bind.item.as_ref() else {
            return;
        };
        let result = InventoryManager::singleton().bind().check_grid_cells(
            item.clone(),
            inventory_id,
            index,
        );
        if let InventoryEntityResult::FreeSpace(cells, _item) = result {
            inventory_grid.bind_mut().highlight_cells(cells);
        } else if let InventoryEntityResult::SpaceTaken(cells, _item) = result {
            inventory_grid.bind_mut().highlight_cells_red(cells);
        }
    }

    fn stop_highlighting_all(&mut self, inventory_ui_manager: &mut InventoryUIManagerView) {
        for mut inventory_grid in inventory_ui_manager.inventories.iter_shared() {
            inventory_grid.bind_mut().stop_highlighting_all();
        }
    }
}

impl InventoryUIManagerState for InventoryUIMoveItemState {
    fn input(
        mut self: Box<Self>,
        event: Gd<InputEvent>,
        mut inventory_ui_manager: InventoryUIManagerView,
    ) -> Box<dyn InventoryUIManagerState> {
        let event_cast = event.try_cast::<InputEventMouseButton>();
        if let Ok(mouse_event) = event_cast {
            if !mouse_event.is_pressed() {
                return self;
            }
            if let Ok(state) =
                self.release_item(mouse_event.get_global_position(), &mut inventory_ui_manager)
            {
                inventory_ui_manager
                    .base
                    .get_viewport()
                    .unwrap()
                    .set_input_as_handled();
                return state;
            }
            return self;
        }

        let Ok(mouse_motion) = event_cast
            .err()
            .unwrap()
            .try_cast::<InputEventMouseMotion>()
        else {
            return self;
        };
        let item_global_pos = self.item_held.get_global_position();
        self.item_held
            .set_global_position(item_global_pos.lerp(mouse_motion.get_global_position(), 0.9));
        self.highlight_grid(mouse_motion.get_global_position(), inventory_ui_manager);
        self
    }

    fn press_event(
        mut self: Box<Self>,
        _presser: Gd<InventoryUIItem>,
        mut inventory_ui_manager: InventoryUIManagerView,
    ) -> Box<dyn InventoryUIManagerState> {
        if inventory_ui_manager
            .cooldown
            .elapsed()
            .unwrap()
            .as_secs_f64()
            > inventory_ui_manager.cooldown_time
        {
            if let Ok(state) = self.release_item(
                inventory_ui_manager.base.get_global_mouse_position(),
                &mut inventory_ui_manager,
            ) {
                return state;
            }
        }
        self
    }

    fn frob_event(
        self: Box<Self>,
        _frobber: Gd<InventoryUIItem>,
        _inventory_ui_manager: InventoryUIManagerView,
    ) -> Box<dyn InventoryUIManagerState> {
        // ignores frob events
        self
    }

    fn hide_event(
        mut self: Box<Self>,
        mut inventory_ui_manager: InventoryUIManagerView,
        is_hidden: bool,
    ) -> Box<dyn InventoryUIManagerState> {
        let mut ui_item_bind = self.item_held.bind_mut();
        let item = ui_item_bind.item.as_mut().unwrap();
        item.emit_signal("moved", &[]);
        drop(ui_item_bind);
        self.stop_highlighting_all(&mut inventory_ui_manager);
        let new_state = Box::new(InventoryHiddenState::default());
        new_state.hide_event(inventory_ui_manager, is_hidden)
    }
}

pub struct InventoryFrobState {
    frobber: Gd<InventoryUIItem>,
    frob_act_react: Gd<ActReactResource>,
}

impl InventoryFrobState {
    fn enter(&self, mut inventory_ui_manager: InventoryUIManagerView) {
        inventory_ui_manager.base.emit_signal(
            "inventory_frob_started",
            &[self.frob_act_react.to_variant()],
        );
    }
    fn exit(&self, inventory_ui_manager: &mut InventoryUIManagerView) {
        inventory_ui_manager
            .base
            .emit_signal("inventory_frob_finished", &[]);
    }
}

impl InventoryUIManagerState for InventoryFrobState {
    fn input(
        self: Box<Self>,
        _event: Gd<InputEvent>,
        _inventory_ui_manager: InventoryUIManagerView,
    ) -> Box<dyn InventoryUIManagerState> {
        self
    }

    fn press_event(
        self: Box<Self>,
        presser: Gd<InventoryUIItem>,
        mut inventory_ui_manager: InventoryUIManagerView,
    ) -> Box<dyn InventoryUIManagerState> {
        if inventory_ui_manager
            .cooldown
            .elapsed()
            .unwrap()
            .as_secs_f64()
            < inventory_ui_manager.cooldown_time
        {
            return self;
        }
        if presser == self.frobber {
            self.exit(&mut inventory_ui_manager);
            return Box::new(InventoryUIDefaultState {});
        }
        let (act_react, context) = {
            let presser_bind = presser.bind();
            let Some(reactor) = presser_bind.item.clone() else {
                return self;
            };
            let item_bind = reactor.bind();
            let Some(Some(act_react)) = item_bind
                .inventory
                .as_ref()
                .map(|i| i.inventory_data.bind().act_react.clone())
            else {
                return self;
            };
            if act_react == self.frob_act_react {
                drop(item_bind);
                drop(presser_bind);
                self.exit(&mut inventory_ui_manager);
                return Box::new(InventoryUIDefaultState {});
            }
            let Some(actor) = self.frobber.bind().item.clone() else {
                return self;
            };
            drop(item_bind);
            let context = dict! {
                "actor": actor,
                "reactor": reactor,
                "inventories": inventory_ui_manager.player_inventory_ids.clone()
            };
            (act_react, context)
        };

        ActReactExecutor::singleton().bind_mut().react(
            self.frob_act_react.clone(),
            act_react,
            context,
        );
        self.exit(&mut inventory_ui_manager);
        Box::new(InventoryUIDefaultState {})
    }

    fn frob_event(
        self: Box<Self>,
        _frobber: Gd<InventoryUIItem>,
        _inventory_ui_manager: InventoryUIManagerView,
    ) -> Box<dyn InventoryUIManagerState> {
        self
    }

    fn hide_event(
        self: Box<Self>,
        mut inventory_ui_manager: InventoryUIManagerView,
        is_hidden: bool,
    ) -> Box<dyn InventoryUIManagerState> {
        self.exit(&mut inventory_ui_manager);
        let new_state = Box::new(InventoryHiddenState::default());
        new_state.hide_event(inventory_ui_manager, is_hidden)
    }
}

#[derive(Default)]
pub struct InventoryHiddenState {}

impl InventoryHiddenState {
    const TWEEN_TIME: f64 = 0.44;
    fn do_tween(
        &self,
        inventory_ui_manager: &mut InventoryUIManagerView,
        is_hidden: bool,
    ) -> Option<Gd<Tween>> {
        let (desired_translation, items_holder_desired_anchors) = if is_hidden {
            (
                inventory_ui_manager.hidden_anchors,
                *inventory_ui_manager.items_holder_initial_anchors
                    + (*inventory_ui_manager.hidden_anchors
                        - *inventory_ui_manager.initial_anchors),
            )
        } else {
            (
                inventory_ui_manager.initial_anchors,
                *inventory_ui_manager.items_holder_initial_anchors,
            )
        };

        if let Some(mut tween) = inventory_ui_manager.tween.take() {
            tween.kill();
        }
        let mut tween = inventory_ui_manager.base.create_tween().unwrap();
        tween
            .tween_property(
                &inventory_ui_manager.base,
                "anchor_top",
                &desired_translation.y.to_variant(),
                Self::TWEEN_TIME,
            )?
            .set_trans(TransitionType::EXPO)?
            .set_ease(EaseType::OUT)?;
        tween
            .parallel()
            .unwrap()
            .tween_property(
                &inventory_ui_manager.base,
                "anchor_bottom",
                &desired_translation.w.to_variant(),
                Self::TWEEN_TIME,
            )?
            .set_trans(TransitionType::EXPO)?
            .set_ease(EaseType::OUT)?;
        tween
            .parallel()
            .unwrap()
            .tween_property(
                &**inventory_ui_manager.items_holder,
                "anchor_top",
                &items_holder_desired_anchors.y.to_variant(),
                Self::TWEEN_TIME,
            )?
            .set_trans(TransitionType::EXPO)?
            .set_ease(EaseType::OUT)?;
        tween
            .parallel()
            .unwrap()
            .tween_property(
                &**inventory_ui_manager.items_holder,
                "anchor_bottom",
                &items_holder_desired_anchors.w.to_variant(),
                Self::TWEEN_TIME,
            )?
            .set_trans(TransitionType::EXPO)?
            .set_ease(EaseType::OUT)?;

        Some(tween)
    }
}

impl InventoryUIManagerState for InventoryHiddenState {
    fn input(
        self: Box<Self>,
        _event: Gd<InputEvent>,
        _inventory_ui_manager: InventoryUIManagerView,
    ) -> Box<dyn InventoryUIManagerState> {
        self
    }

    fn press_event(
        self: Box<Self>,
        _presser: Gd<InventoryUIItem>,
        _inventory_ui_manager: InventoryUIManagerView,
    ) -> Box<dyn InventoryUIManagerState> {
        self
    }

    fn frob_event(
        self: Box<Self>,
        _frobber: Gd<InventoryUIItem>,
        _inventory_ui_manager: InventoryUIManagerView,
    ) -> Box<dyn InventoryUIManagerState> {
        self
    }

    fn hide_event(
        self: Box<Self>,
        mut inventory_ui_manager: InventoryUIManagerView,
        is_hidden: bool,
    ) -> Box<dyn InventoryUIManagerState> {
        if is_hidden {
            *inventory_ui_manager.tween = self.do_tween(&mut inventory_ui_manager, is_hidden);
            return self;
        }
        *inventory_ui_manager.tween = self.do_tween(&mut inventory_ui_manager, is_hidden);
        Box::new(InventoryUIDefaultState)
    }
}
