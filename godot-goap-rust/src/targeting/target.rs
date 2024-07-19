use godot::obj::InstanceId;
use godot::prelude::Vector3;
use serde::{Deserialize, Serialize};
use strum_macros::EnumDiscriminants;


/// todo – use some generational id instead of instances ids

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(name(TargetType))]
#[strum_discriminants(derive(Serialize, Deserialize, Hash))]
pub enum AITarget {
    /// some character target, like a player, with a hitbox position attached
    Character(InstanceId, Option<Vector3>),
    /// A combat opportunity node
    CombatOpportunity,
    /// a point of disturbance (noise)
    Disturbance(Vector3),
    /// a point of interest
    Interest(Vector3),
    /// object, like doors, exploding barrels etc
    Object(InstanceId),
    /// interactable
    SmartObject(InstanceId),
}

impl AITarget {
    pub fn get_target_pos(&self) -> Option<Vector3> {
        match self {
            AITarget::Character(_, pos) => {*pos}
            AITarget::Disturbance(pos) | AITarget::Interest(pos) => {Some(*pos)}
            AITarget::CombatOpportunity | AITarget::Object(_) | AITarget::SmartObject(_) => {None}
        }
    }
}