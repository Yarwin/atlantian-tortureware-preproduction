use godot::prelude::*;
use crate::act_react::game_effect::{EffectResult, GameEffect, GameEffectProcessor};
use crate::act_react::game_effect_builder::{GameEffectInitializer, register_effect_builder};
use crate::godot_api::gamesys::GameSystem;
use crate::godot_api::inventory_manager::InventoryManager;
use crate::godot_api::item_object::Item;


#[derive(GodotClass, Debug)]
#[class(base=Resource)]
pub struct PickupItemGameEffect {
    base: Base<Resource>
}

#[godot_api]
impl IResource for PickupItemGameEffect {
    fn init(base: Base<Self::Base>) -> Self {
        register_effect_builder::<Self>("PickupItemGameEffect".into());
        PickupItemGameEffect{ base }
    }
}

#[godot_api]
impl PickupItemGameEffect {
    #[func]
    pub fn builder_name(&self) -> StringName {
        "PickupItemGameEffect".into()
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

impl GameEffect for PickupItem {
    fn execute(&mut self) -> EffectResult {
        let mut inventory_manager = InventoryManager::singleton();
        let mut item = self.item.take().unwrap();
        for inventory_id in self.inventories_ids.iter_shared() {
            let result = inventory_manager.bind_mut().put_item_at_first_free_space(item, None, inventory_id);
            match result {
                Ok(_) => {return EffectResult::Free},
                Err(e) => {item = e.item()}
            }
        }
        EffectResult::Free
    }
}
