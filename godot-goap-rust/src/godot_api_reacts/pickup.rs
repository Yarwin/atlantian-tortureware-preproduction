use godot::prelude::*;
use crate::act_react::game_effect::{EffectResult, GameEffect, GameEffectProcessor};
use crate::act_react::game_effect_builder::{GameEffectInitializer, register_effect_builder};
use crate::godot_api::gamesys::GameSystem;
use crate::godot_api::inventory_manager::InventoryManager;
use crate::godot_api::item_object::Item;
use crate::inventory::inventory_item::StackResult;


#[derive(GodotClass, Debug)]
#[class(base=Resource)]
pub struct PickupItemGameEffect {
    base: Base<Resource>
}

#[godot_api]
impl IResource for PickupItemGameEffect {
    fn init(base: Base<Self::Base>) -> Self {
        register_effect_builder::<Self>(Self::class_name().to_gstring());
        PickupItemGameEffect{ base }
    }
}

impl GameEffectInitializer for PickupItemGameEffect {
    fn build(&self, _act: &Dictionary, context: &Dictionary) -> GameEffectProcessor {
        let Some(reactor) = context.get("reactor").map(|v| v.to::<Gd<Node>>()) else {panic!("no reactor!")};

        let Ok(item) = reactor.get("item".into()).try_to::<Gd<Item>>() else {panic!("no item to pickup!")};

        let Some(inventories) = context.get("inventories").map(|v| v.to::<Array<u32>>()) else {panic!("no inventories to put item in!")};
        let pickup_item = PickupItem {
            item: Some(item),
            inventories_ids: inventories,
        };
        let obj = Gd::from_object(pickup_item);
        GameEffectProcessor::new(obj)
    }
}

#[derive(GodotClass, Debug)]
#[class(init, base=Object)]
pub struct PickupItem {
    pub item: Option<Gd<Item>>,
    pub inventories_ids: Array<u32>
}

impl PickupItem {
    fn try_stack_item(&mut self, item: Gd<Item>) -> Result<EffectResult, Gd<Item>> {
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
            let items = inventory_manager.bind().get_items_of_the_same_type(inventory_id, stack_item_data.clone());
            for other_item in items.iter_shared() {
                match inventory_manager.bind_mut().try_stack_item(item.clone(), other_item) {
                    StackResult::WrongType | StackResult::NoChange | StackResult::Updated => continue,
                    StackResult::Depleted => return Ok(EffectResult::Free)
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
        match self.try_stack_item(item) {
            Ok(result) => return result,
            Err(i) => {item = i}
        }
        self.try_place_item(item).unwrap_or(EffectResult::Free)
    }
}
