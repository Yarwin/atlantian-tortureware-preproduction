use crate::act_react::act_react_resource::Reaction;
use crate::act_react::game_effect::{EffectResult, GameEffect};
use crate::godot_api::gamesys::GameSystem;
use crate::godot_api::godot_inventory::ItemToSpawn;
use crate::godot_api::inventory_manager::InventoryManager;
use crate::godot_api::item_object::Item;
use crate::godot_api_reacts::print_message::PrintMessage;
use godot::classes::Resource;
use godot::prelude::*;

#[derive(GodotClass)]
#[class(init, base=Resource)]
pub struct CombineInventoryItemGameEffect {
    /// dictionary that holds combiner StringNames as keys, and items to create as values
    // todo - change it to individual resources?
    #[export]
    pub on_combine: Dictionary,
    base: Base<Resource>,
}

impl CombineInventoryItemGameEffect {
    fn on_failure(&self) -> Option<DynGd<Object, dyn GameEffect>> {
        // return information about failure. Right now it just prints info in console
        // change it to sad fart sound in the future.
        let print_message = PrintMessage {
            message: GString::from("Can't combine!"),
        };
        let obj = Gd::from_object(print_message);
        Some(obj.into_dyn::<dyn GameEffect>().upcast())
    }
}

#[godot_dyn]
impl Reaction for CombineInventoryItemGameEffect {
    fn can_react(&self, context: &Dictionary) -> bool {
        let Some(combine_name) = context.get("combinator").map(|v| v.to::<StringName>()) else {
            return false;
        };
        if self.on_combine.get(combine_name).is_some() {
            return true;
        }
        false
    }

    fn build_effect(
        &self,
        act_context: &Dictionary,
        context: &Dictionary,
    ) -> Option<DynGd<Object, dyn GameEffect>> {
        let (Some(actor), Some(reactor), Some(inventories_ids), Some(combine_name), reduce_stack) = (
            context.get("actor").map(|v| v.to::<Gd<Item>>()),
            context.get("reactor").map(|v| v.to::<Gd<Item>>()),
            context.get("inventories").map(|va| va.to::<Array<u32>>()),
            context.get("combinator").map(|v| v.to::<StringName>()),
            act_context
                .get("reduce_stack")
                .map(|v| v.to::<bool>())
                .unwrap_or(true),
        ) else {
            return self.on_failure();
        };
        // let Some(actor) = context.get("actor").map(|v| v.to::<Gd<Item>>()) else {
        //     return self.on_failure();
        // };
        // let Some(reactor) = context.get("reactor").map(|v| v.to::<Gd<Item>>()) else {
        //     return self.on_failure();
        // };
        // let Some(inventories_ids) = context.get("inventories").map(|va| va.to::<Array<u32>>())
        //     else {
        //         return self.on_failure();
        //     };
        // let Some(combine_name) = act_context.get("combinator").map(|v| v.to::<StringName>()) else {
        //     return self.on_failure();
        // };
        let Some(combine_outcome) = self
            .on_combine
            .get(combine_name)
            .map(|v| v.to::<Gd<ItemToSpawn>>())
        else {
            return self.on_failure();
        };
        // let reduce_stack = act_context
        //     .get("reduce_stack")
        //     .map(|v| v.to::<bool>())
        //     .unwrap_or(true);
        let effect = CombineItemsInInventory {
            actor: Some(actor),
            reactor: Some(reactor),
            reduce_stack,
            inventories_ids,
            outcome: Some(combine_outcome),
        };
        let obj = Gd::from_object(effect);
        Some(obj.into_dyn::<dyn GameEffect>().upcast())
    }
}

#[derive(GodotClass)]
#[class(init, base=Object)]
pub struct CombineItemsInInventory {
    actor: Option<Gd<Item>>,
    reactor: Option<Gd<Item>>,
    #[init(val = true)]
    reduce_stack: bool,
    inventories_ids: Array<u32>,
    outcome: Option<Gd<ItemToSpawn>>,
}

#[godot_dyn]
impl GameEffect for CombineItemsInInventory {
    fn execute(&mut self) -> EffectResult {
        let mut inventory_manager = InventoryManager::singleton();
        if self.reduce_stack {
            inventory_manager
                .bind_mut()
                .reduce_stack(self.actor.take().unwrap(), 1);
        }
        inventory_manager
            .bind_mut()
            .reduce_stack(self.reactor.take().unwrap(), 1);
        let mut was_item_created: bool = false;
        for inventory_id in self.inventories_ids.iter_shared() {
            if inventory_manager
                .bind_mut()
                .create_item_in_inventory(self.outcome.clone().unwrap(), inventory_id)
            {
                was_item_created = true;
                break;
            }
        }
        if !was_item_created {
            // todo – create & drop the item if there is no space in the inventory. Might require more context.
            godot_print!("item was not created!");
        }
        EffectResult::Free
    }
}
