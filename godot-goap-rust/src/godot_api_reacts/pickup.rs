use godot::prelude::*;
use godot::classes::{Resource};
use crate::act_react::game_effect::{EffectResult, GameEffect, GameEffectProcessor};
use crate::act_react::game_effect_builder::GameEffectInitializer;
use crate::godot_api::gamesys::{GameSys, GameSystem};
use crate::godot_api::inventory_manager::InventoryManager;
use crate::godot_api::item_object::Item;
use crate::inventory::inventory_item::StackResult;


#[derive(GodotClass, Debug)]
#[class(init, base=Resource)]
pub struct PickupItemGameEffect {
    base: Base<Resource>
}


#[godot_api]
impl PickupItemGameEffect {
    #[func]
    fn get_react_display(&self) -> GString {
        GString::from("Pickup item")
    }
}


impl GameEffectInitializer for PickupItemGameEffect {
    fn build(&self, _act_context: &Dictionary, context: &Dictionary) -> Option<GameEffectProcessor> {
        let reactor = context.get("reactor").map(|v| v.to::<Gd<Node>>())?;

        let item = reactor.get("item".into()).try_to::<Gd<Item>>().ok()?;

        let inventories = context.get("inventories").map(|v| v.to::<Array<u32>>())?;
        let pickup_item = PickupItem {
            item: Some(item),
            inventories_ids: inventories,
        };
        let obj = Gd::from_object(pickup_item);
        Some(GameEffectProcessor::new(obj))
    }
}

#[derive(GodotClass, Debug)]
#[class(init, base=Object)]
pub struct PickupItem {
    pub item: Option<Gd<Item>>,
    pub inventories_ids: Array<u32>
}

impl PickupItem {
    fn try_stack_item(&mut self, item: Gd<Item>, _success_message: &str) -> Result<EffectResult, Gd<Item>> {
        let mut inventory_manager = InventoryManager::singleton();
        let potential_inventory_data = {
            let item_bind = item.bind();
            // bail if item has no inventory component and can't be picked up
            let Some(inventory_data) = item_bind.inventory.as_ref().map(|i| i.inventory_data.clone()) else {return Ok(EffectResult::Free)};
            if inventory_data.bind().max_stack > 1 {
                Some(inventory_data)
            } else {
                None
            }
        };

        let Some(stack_item_data) = potential_inventory_data else {return Err(item)};

        for inventory_id in self.inventories_ids.iter_shared() {
            let mut inventory_agent = inventory_manager.bind().get_inventory_agent(inventory_id);
            let items = inventory_manager.bind().get_items_of_the_same_type(inventory_id, stack_item_data.clone());
            for other_item in items.iter_shared() {
                match inventory_manager.bind_mut().try_stack_item(item.clone(), other_item.clone()) {
                     StackResult::Updated => {
                         if let Some(inventory_agent) = inventory_agent.as_mut() {
                             inventory_agent.emit_signal("stack_updated".into(), &[other_item.to_variant()]);
                         }
                         continue
                     },
                    StackResult::WrongType | StackResult::NoChange => continue,
                    StackResult::Depleted => {
                        if let Some(inventory_agent) = inventory_agent.as_mut() {
                            inventory_agent.emit_signal("stack_updated".into(), &[other_item.to_variant()]);
                        }

                        return Ok(EffectResult::Free);
                    }
                }
            }
        }
        Err(item)
    }

    fn try_place_item(&mut self, mut item: Gd<Item>) -> Result<EffectResult, Gd<Item>> {
        let mut inventory_manager = InventoryManager::singleton();
        for inventory_id in self.inventories_ids.iter_shared() {
            let result = inventory_manager.bind_mut().put_item_at_first_free_space(item, None, inventory_id);
            match result {
                Ok(_) => {return Ok(EffectResult::Free)},
                Err(e) => {item = e.item()}
            }
        }
        Err(item)
    }
}

impl GameEffect for PickupItem {
    fn execute(&mut self) -> EffectResult {
        let mut item = self.item.take().unwrap();
        let initial_stack = {
            let item_bind = item.bind();
            item_bind.inventory.as_ref().unwrap().stack
        };
        let generate_message = |item: &Gd<Item>, stack: u32| {
            let item_bind = item.bind();
            let item_name = item_bind.item_resource.as_ref().unwrap().bind().name.to_string();
            if stack > 1 {
                format!("Picked up {stack} {}", item_name)
            } else {
                format!("Picked up {}", item_name)
            }
        };
        let success_message = generate_message(&item, initial_stack);
        match self.try_stack_item(item, &success_message) {
            Ok(result) => {
                if result == EffectResult::Free {
                    GameSys::singleton().emit_signal("new_log_message".into(), &[success_message.to_variant()]);
                }
                return result;
            },
            Err(i) => {item = i}
        }
        match self.try_place_item(item) {
            Ok(e) => {
                GameSys::singleton().emit_signal("new_log_message".into(), &[success_message.to_variant()]);
                e
            },
            Err(i) => {
                let stack_diff = {
                    let item_bind = i.bind();
                    initial_stack - item_bind.inventory.as_ref().unwrap().stack
                };

                if stack_diff != 0 {
                    let partial_success_message = generate_message(&i, stack_diff);
                    GameSys::singleton().emit_signal("new_log_message".into(), &[partial_success_message.to_variant()]);
                }
                EffectResult::Free
            }
        }
    }
}
