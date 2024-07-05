use std::sync::{Arc, Mutex};
use godot::obj::InstanceId;
use godot::prelude::Vector3;
use serde::{Serialize, Deserialize};
use strum_macros::EnumDiscriminants;
use crate::ai_nodes::ai_node::AINode;

#[derive(Debug)]
#[derive(EnumDiscriminants)]
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
    /// patrol point
    PatrolPoint(Arc<Mutex<AINode>>, Vector3),
}

