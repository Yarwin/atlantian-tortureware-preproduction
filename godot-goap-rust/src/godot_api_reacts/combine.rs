use godot::classes::Resource;
use godot::prelude::*;
use crate::act_react::game_effect::{EffectResult, GameEffect, GameEffectProcessor};
use crate::act_react::game_effect_builder::{GameEffectInitializer, register_effect_builder};
use crate::godot_api::godot_inventory::ItemToSpawn;
use crate::godot_api::inventory_manager::InventoryManager;
use crate::godot_api::item_object::{Item};
use crate::godot_api_reacts::print_message::PrintMessage;
use crate::godot_api::gamesys::GameSystem;

#[derive(GodotClass, Debug)]
#[class(base=Resource)]
pub struct CombineInventoryItemGameEffect {
    /// dictionary that holds combiner StringNames as keys, and items to create as values
    #[export]
    pub on_combine: Dictionary,
    base: Base<Resource>,
}


impl CombineInventoryItemGameEffect {
    fn on_failure(&self) -> GameEffectProcessor {
        // return information about failure. Right now it just prints info in console
        // change it to sad fart sound in the future.
        let print_message = PrintMessage {
            message: GString::from("Can't combine!")
        };
        let obj = Gd::from_object(print_message);
        GameEffectProcessor::new(obj)
    }
}

#[godot_api]
impl IResource for CombineInventoryItemGameEffect {
    fn init(base: Base<Self::Base>) -> Self {
        register_effect_builder::<Self>("CombineInventoryItemGameEffect".into());
        CombineInventoryItemGameEffect {
            on_combine: Dictionary::new(),
            base
        }
    }
}

#[godot_api]
impl CombineInventoryItemGameEffect {
    #[func]
    fn can_react(&self, act_context: Dictionary) -> bool {
        let Some(combine_name) = act_context.get("combinator").map(|v| v.to::<StringName>()) else {return false};
        if self.on_combine.get(combine_name).is_some() {
            return true
        }
        false
    }

    #[func]
    pub fn builder_name(&self) -> StringName {
        "CombineInventoryItemGameEffect".into()
    }
}

impl GameEffectInitializer for CombineInventoryItemGameEffect {
    fn build(&self, act_context: &Dictionary, context: &Dictionary) -> GameEffectProcessor {
        let Some(actor) = context.get("actor").map(|v| v.to::<Gd<Item>>()) else {return self.on_failure()};
        let Some(reactor) = context.get("reactor").map(|v| v.to::<Gd<Item>>()) else {return self.on_failure()};
        let Some(inventories_ids) = context.get("inventories").map(|va| va.to::<Array<u32>>()) else {return self.on_failure()};
        let Some(combine_name) = act_context.get("combinator").map(|v| v.to::<StringName>()) else {return self.on_failure()};
        let Some(combine_outcome) = self.on_combine.get(combine_name).map(|v| v.to::<Gd<ItemToSpawn>>()) else {return self.on_failure()};
        let reduce_stack = act_context.get("reduce_stack").map(|v| v.to::<bool>()).unwrap_or(true);
        let combine_items = CombineItemsInInventory {
            actor: Some(actor),
            reactor: Some(reactor),
            reduce_stack,
            inventories_ids,
            outcome: Some(combine_outcome),
        };
        GameEffectProcessor::new(Gd::from_object(combine_items))
    }
}

#[derive(GodotClass, Debug)]
#[class(init, base=Object)]
pub struct CombineItemsInInventory {
    actor: Option<Gd<Item>>,
    reactor: Option<Gd<Item>>,
    #[init(default = true)]
    reduce_stack: bool,
    inventories_ids: Array<u32>,
    outcome: Option<Gd<ItemToSpawn>>,
}

impl GameEffect for CombineItemsInInventory {
    fn execute(&mut self) -> EffectResult {
        let mut inventory_manager = InventoryManager::singleton();
        if self.reduce_stack {
            inventory_manager.bind_mut().reduce_stack(self.actor.take().unwrap(), 1);
        }
        inventory_manager.bind_mut().reduce_stack(self.reactor.take().unwrap(), 1);
        for inventory_id in self.inventories_ids.iter_shared() {
            if inventory_manager.bind_mut().create_item(self.outcome.clone().unwrap(), inventory_id) {
                break
            }
        }
        EffectResult::Free
    }
}