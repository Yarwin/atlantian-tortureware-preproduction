use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};
use std::ops::{Index, IndexMut};

use crate::targeting::target::TargetType;
use serde::{Deserialize, Serialize};
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
pub enum DistanceToTargetType {
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
    DistanceToTarget(DistanceToTargetType),
    Target(TargetType),
}

/// An abstraction that keeps symbolic representation of the world
/// for planing purposes
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash, EnumIter)]
pub enum WorldStateProperty {
    AmILookingAtTarget,
    AnimLooped,
    AnimPlayed,
    AtNode,
    AtNodeType,
    AtTargetPosition,
    CoverStatus,
    DistanceToTarget,
    HasTarget,
    IsAreaSurveyed,
    IsIdling,
    IsInCombat,
    IsNavigationFinished,
    IsPositionValid,
    IsTargetAimingAtMe,
    IsTargetDead,
    IsTargetLookingAtMe,
    IsWeaponArmed,
    IsWeaponLoaded,
    ReactedToWorldStateEvent,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WorldState {
    pub i_am_looking_at_target: Option<WSProperty>,
    pub anim_looped: Option<WSProperty>,
    pub anim_played: Option<WSProperty>,
    pub at_node: Option<WSProperty>,
    pub at_node_type: Option<WSProperty>,
    pub at_target_position: Option<WSProperty>,
    pub cover_status: Option<WSProperty>,
    pub distance_to_target: Option<WSProperty>,
    pub has_target: Option<WSProperty>,
    pub is_area_surveyed: Option<WSProperty>,
    pub is_idling: Option<WSProperty>,
    pub is_in_combat: Option<WSProperty>,
    pub is_navigation_finished: Option<WSProperty>,
    pub is_position_valid: Option<WSProperty>,
    pub is_target_aiming_at_me: Option<WSProperty>,
    pub is_target_dead: Option<WSProperty>,
    pub is_target_looking_at_me: Option<WSProperty>,
    pub is_weapon_armed: Option<WSProperty>,
    pub is_weapon_loaded: Option<WSProperty>,
    pub reacted_to_world_state_event: Option<WSProperty>,
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

impl Index<WorldStateProperty> for WorldState {
    type Output = Option<WSProperty>;

    fn index(&self, index: WorldStateProperty) -> &Self::Output {
        match index {
            WorldStateProperty::AmILookingAtTarget => &self.i_am_looking_at_target,
            WorldStateProperty::AnimLooped => &self.anim_looped,
            WorldStateProperty::AnimPlayed => &self.anim_played,
            WorldStateProperty::ReactedToWorldStateEvent => &self.reacted_to_world_state_event,
            WorldStateProperty::AtNode => &self.at_node,
            WorldStateProperty::AtNodeType => &self.at_node_type,
            WorldStateProperty::CoverStatus => &self.cover_status,
            WorldStateProperty::DistanceToTarget => &self.distance_to_target,
            WorldStateProperty::HasTarget => &self.has_target,
            WorldStateProperty::IsInCombat => &self.is_in_combat,
            WorldStateProperty::IsIdling => &self.is_idling,
            WorldStateProperty::IsPositionValid => &self.is_position_valid,
            WorldStateProperty::IsAreaSurveyed => &self.is_area_surveyed,
            WorldStateProperty::IsTargetAimingAtMe => &self.is_target_looking_at_me,
            WorldStateProperty::IsTargetLookingAtMe => &self.is_target_aiming_at_me,
            WorldStateProperty::IsTargetDead => &self.is_target_dead,
            WorldStateProperty::AtTargetPosition => &self.at_target_position,
            WorldStateProperty::IsWeaponArmed => &self.is_weapon_armed,
            WorldStateProperty::IsWeaponLoaded => &self.is_weapon_loaded,
            WorldStateProperty::IsNavigationFinished => &self.is_navigation_finished,
        }
    }
}

impl IndexMut<WorldStateProperty> for WorldState {
    fn index_mut(&mut self, index: WorldStateProperty) -> &mut Self::Output {
        match index {
            WorldStateProperty::AmILookingAtTarget => &mut self.i_am_looking_at_target,
            WorldStateProperty::AnimLooped => &mut self.anim_looped,
            WorldStateProperty::AnimPlayed => &mut self.anim_played,
            WorldStateProperty::ReactedToWorldStateEvent => &mut self.reacted_to_world_state_event,
            WorldStateProperty::AtNode => &mut self.at_node,
            WorldStateProperty::AtNodeType => &mut self.at_node_type,
            WorldStateProperty::CoverStatus => &mut self.cover_status,
            WorldStateProperty::DistanceToTarget => &mut self.distance_to_target,
            WorldStateProperty::HasTarget => &mut self.has_target,
            WorldStateProperty::IsInCombat => &mut self.is_in_combat,
            WorldStateProperty::IsIdling => &mut self.is_idling,
            WorldStateProperty::IsPositionValid => &mut self.is_position_valid,
            WorldStateProperty::IsAreaSurveyed => &mut self.is_area_surveyed,
            WorldStateProperty::IsTargetAimingAtMe => &mut self.is_target_looking_at_me,
            WorldStateProperty::IsTargetLookingAtMe => &mut self.is_target_aiming_at_me,
            WorldStateProperty::IsTargetDead => &mut self.is_target_dead,
            WorldStateProperty::AtTargetPosition => &mut self.at_target_position,
            WorldStateProperty::IsWeaponArmed => &mut self.is_weapon_armed,
            WorldStateProperty::IsWeaponLoaded => &mut self.is_weapon_loaded,
            WorldStateProperty::IsNavigationFinished => &mut self.is_navigation_finished,
        }
    }
}

impl Hash for WorldState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for k in WorldStateProperty::iter() {
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
            if let Some(p) = other[k].clone() {
                self[k] = Some(p);
            }
        }
    }

    /// Return the number of discrepancies between properties of WorldStates.
    pub fn count_state_differences(&self, other: &WorldState) -> u32 {
        let mut count: u32 = 0;
        for key in WorldStateProperty::iter() {
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
