use godot::obj::InstanceId;
use godot::prelude::Vector3;
use serde::{Deserialize, Serialize};
use strum_macros::EnumDiscriminants;

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(name(TargetType))]
#[strum_discriminants(derive(Serialize, Deserialize, Hash))]
pub enum Target {
    /// some character target, like a player
    Character(InstanceId),
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
