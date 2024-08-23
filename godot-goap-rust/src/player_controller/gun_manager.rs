use std::collections::HashMap;
use godot::prelude::*;
use crate::equipment::equip_component::EquipmentComponent;

pub struct GunManager {
    // holds builder resource & initialized nodes
    pub initialized: HashMap<Gd<Resource>, EquipmentComponent>
}
