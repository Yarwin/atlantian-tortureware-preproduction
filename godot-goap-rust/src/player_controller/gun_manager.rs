use crate::equipment::equip_component::EquipmentComponent;
use godot::prelude::*;
use std::collections::HashMap;

pub struct GunManager {
    // holds builder resource & initialized nodes
    pub initialized: HashMap<Gd<Resource>, EquipmentComponent>,
}
