use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Index, IndexMut};

use crate::targeting::target::TargetType;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub enum AIWorldStateEvent {
    None,
    Block,
    BlockedPath,
    Damage,
    EnemyInPlaceForSurpriseAttack,
    Stunned,
    Staggered,
    Surprised
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeTypeEnum {
    Ambush,
    Cover,
    Guard,
    Interest,
    PatrolPoint,
    Position,
    SmartObject,
    View,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub enum CoverStatusType {
    /// in cover
    Covered,
    /// in open
    Exposed,
}

/// enum representing relative combat distance to given target
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub enum DistanceToTarget {
    /// melee attack range
    Close,
    Medium,
    Far,
    /// enemy can't be attacked
    OutsideReach,
}


#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub enum WSProperty {
    String(String),
    Truth(bool),
    WorldStateEvent(AIWorldStateEvent),
    Node(NodeTypeEnum),
    CoverStatus(CoverStatusType),
    DistanceToTarget(DistanceToTarget),
    Target(TargetType),
}


/// An abstraction that keeps symbolic representation of the world
/// for planing purposes
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash, EnumIter)]
pub enum WorldStateProperty {
    AmILookingAtTarget = 0,
    AnimLooped,
    AnimPlayed,
    AtNode,
    AtNodeType,
    AtTargetPosition,
    CoverStatus,
    DistanceToTarget,
    HasTarget,
    HasAttack,
    IsAreaSurveyed,
    IsDead,
    IsIdling,
    IsInCombat,
    IsNavigationFinished,
    IsPositionValid,
    IsTargetAimingAtMe,
    IsTargetDead,
    IsTargetLookingAtMe,
    IsWeaponArmed,
    IsWeaponLoaded,
    IsRecoveringFromAttack,
    ReactedToWorldStateEvent,
    Max
}

#[derive(Clone, Default)]
pub struct WorldState {
    // pub i_am_looking_at_target: Option<WSProperty>,
    // pub anim_looped: Option<WSProperty>,
    // pub anim_played: Option<WSProperty>,
    // pub at_node: Option<WSProperty>,
    // pub at_node_type: Option<WSProperty>,
    // pub at_target_position: Option<WSProperty>,
    // pub cover_status: Option<WSProperty>,
    // pub distance_to_target: Option<WSProperty>,
    // pub has_target: Option<WSProperty>,
    // pub is_area_surveyed: Option<WSProperty>,
    // pub is_dead: Option<WSProperty>,
    // pub is_idling: Option<WSProperty>,
    // pub is_in_combat: Option<WSProperty>,
    // pub is_navigation_finished: Option<WSProperty>,
    // pub is_position_valid: Option<WSProperty>,
    // pub is_target_aiming_at_me: Option<WSProperty>,
    // pub is_target_dead: Option<WSProperty>,
    // pub is_target_looking_at_me: Option<WSProperty>,
    // pub is_weapon_armed: Option<WSProperty>,
    // pub is_weapon_loaded: Option<WSProperty>,
    // pub is_recovering_from_attack: Option<WSProperty>,
    // pub reacted_to_world_state_event: Option<WSProperty>,
    pub inner: [Option<WSProperty>; WorldStateProperty::Max as usize]
}


impl Serialize for WorldState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let map_size = self
            .inner
            .iter()
            .fold(
                0usize,
                |mut a, b| {
                    if b.is_some() {
                        a += 1;
                    }
                    a
                });
        let mut map = serializer.serialize_map(Some(map_size))?;
        for (idx, p) in self.inner.iter().enumerate() {
            if let Some(property) = p {
                let k = unsafe {std::mem::transmute::<u8, WorldStateProperty>(idx as u8)};
                map.serialize_entry(&k, property)?;
            }
        }
        map.end()
    }
}

struct WorldStateVisitor;

impl<'de> Visitor<'de> for WorldStateVisitor {
    type Value = WorldState;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("an integer between -2^31 and 2^31")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where A: MapAccess<'de>
    {
        let mut new_map: HashMap<WorldStateProperty, WSProperty> = HashMap::new();
        while let Some((key, value)) = map.next_entry()? {
            new_map.insert(key, value);
        }
        Ok(WorldState::from(new_map))
    }
}

impl<'de> Deserialize<'de> for WorldState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_map(WorldStateVisitor {})

    }
}

impl From<HashMap<WorldStateProperty, WSProperty>> for WorldState {
    fn from(value: HashMap<WorldStateProperty, WSProperty>) -> Self {
        let mut world_state = Self::default();
        for (k, v) in value.into_iter() {
            world_state[k] = Some(v);
        }
        world_state
    }
}

impl<const N: usize> From<[(WorldStateProperty, WSProperty); N]> for WorldState {
    fn from(arr: [(WorldStateProperty, WSProperty); N]) -> Self {
        let mut state = WorldState::default();
        for (key, value) in arr {
            state[key] = Some(value);
        }
        state
    }
}

impl Debug for WorldState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut output = String::from("WorldState: ");
        for key in WorldStateProperty::iter() {
            if key == WorldStateProperty::Max {break}
            if self[key].is_some() {
                output.push_str(&format!("{:?}: {:?}; ", key, self[key].as_ref().unwrap()))
            }
        }
        write!(f, "{}", output)
    }
}

impl Index<WorldStateProperty> for WorldState {
    type Output = Option<WSProperty>;

    fn index(&self, index: WorldStateProperty) -> &Self::Output {
        &self.inner[index as usize]
    }
}

impl IndexMut<WorldStateProperty> for WorldState {
    fn index_mut(&mut self, index: WorldStateProperty) -> &mut Self::Output {
        &mut self.inner[index as usize]
    }
}

impl Hash for WorldState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for k in WorldStateProperty::iter() {
            if k == WorldStateProperty::Max {break}
            self[k].hash(state);
        }
    }
}

impl WorldState {
    pub fn get_world_state_property(&self, value: WorldStateProperty) -> Option<WSProperty> {
        if let Some(property) = self[value].clone() {
            return Some(property);
        }
        None
    }

    /// insert other worldstate properties into its state
    pub fn apply_world_state(&mut self, other: &WorldState) {
        for k in WorldStateProperty::iter() {
            if k == WorldStateProperty::Max {break}
            if let Some(p) = other[k].clone() {
                self[k] = Some(p);
            }
        }
    }

    /// Return the number of discrepancies between properties of WorldStates.
    pub fn count_state_differences(&self, other: &WorldState) -> u32 {
        let mut count: u32 = 0;
        for key in WorldStateProperty::iter() {
            if key == WorldStateProperty::Max {break}
            let (property, other_property) = (self[key].as_ref(), other[key].as_ref());
            if let Some(other_val) = property {
                if let Some(equality) = other_property.map(|v| v == other_val) {
                    if equality {
                        continue;
                    }
                }
            }
            if property.is_some() || other_property.is_some() {
                count += 1;
            }
        }
        count
    }

    /// Return the number of unsatisfied props in this world state, relative to another.
    pub fn count_unsatisfied_world_state_props(&self, other: &WorldState) -> u32 {
        let mut count: u32 = 0;
        for key in WorldStateProperty::iter() {
            if key == WorldStateProperty::Max {break}
            if let Some(property) = self[key].as_ref() {
                let are_props_equal = other[key].as_ref().map(|other_property| other_property == property).unwrap_or(false);
                if are_props_equal {
                    continue
                }

                count += 1;
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::world_state::WSProperty::Truth;
    #[test]
    fn test_get_prop() {
        let state = WorldState::from([(WorldStateProperty::IsIdling, Truth(true))]);
        assert_eq!(state[WorldStateProperty::IsIdling], Some(Truth(true)));
    }

    #[test]
    fn test_set_prop() {
        let mut state = WorldState::default();
        state[WorldStateProperty::IsIdling] = Some(Truth(true));
        assert_eq!(state[WorldStateProperty::IsIdling], Some(Truth(true)))
    }

    #[test]
    fn test_apply_world_state() {
        let goal_state = WorldState::from([
            (WorldStateProperty::IsIdling, Truth(true)),
            (WorldStateProperty::IsPositionValid, Truth(false)),
        ]);
        let mut state_1 = WorldState::from([(WorldStateProperty::IsIdling, Truth(true))]);
        assert_eq!(goal_state.count_unsatisfied_world_state_props(&state_1), 1);
        let state_2 = WorldState::from([(WorldStateProperty::IsPositionValid, Truth(false))]);
        state_1.apply_world_state(&state_2);
        assert_eq!(goal_state.count_unsatisfied_world_state_props(&state_1), 0);
    }

    #[test]
    fn test_state_differences() {
        let state = WorldState::from([
            (WorldStateProperty::IsIdling, Truth(true)),
            (WorldStateProperty::IsPositionValid, Truth(false)),
            (WorldStateProperty::IsTargetAimingAtMe, Truth(true)),
        ]);
        let other_state = WorldState::from([
            (WorldStateProperty::IsIdling, Truth(false)),
            (WorldStateProperty::IsPositionValid, Truth(false)),
            (WorldStateProperty::IsAreaSurveyed, Truth(true)),
        ]);
        assert_eq!(3, state.count_state_differences(&other_state));
        assert_eq!(2, state.count_unsatisfied_world_state_props(&other_state));
    }
}
