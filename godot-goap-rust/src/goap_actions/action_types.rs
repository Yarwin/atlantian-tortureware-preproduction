use std::collections::HashMap;
use crate::goap_actions::action_component::ActionComponent;
use crate::ai::blackboard::Blackboard;
use crate::ai::working_memory::WorkingMemory;
use crate::ai::world_state::WorldState;
use crate::animations::animation_data::AnimationsData;
use godot::builtin::Rid;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use enum_dispatch::enum_dispatch;
use strum_macros::EnumDiscriminants;
use crate::ai_nodes::ai_node::AINode;
use crate::goap_actions::aim_action::AimWeapon;
use crate::goap_actions::animate_action::Animate;
use crate::goap_actions::attack_ranged_action::RangedAttack;
use crate::goap_actions::arm_weapon_action::ArmWeapon;
use crate::goap_actions::goto_action::GoTo;
use crate::goap_actions::patrol_action::Patrol;
use crate::goap_actions::release_weapon_action::ReleaseWeapon;


#[derive(Debug)]
pub struct AgentActionWorldContext<'a> {
    pub working_memory: &'a WorkingMemory,
    pub current_world_state: &'a mut WorldState,
    pub blackboard: &'a mut Blackboard,
    pub navigation_map_rid: Option<Rid>,
    pub animations: &'a Arc<AnimationsData>,
    pub ai_nodes: &'a mut Option<Arc<RwLock<HashMap<u32, AINode>>>>,
}

#[derive(Debug)]
pub struct AgentActionPlanContext<'a> {
    pub working_memory: &'a mut WorkingMemory,
    pub current_world_state: &'a mut WorldState,
    pub blackboard: &'a mut Blackboard,
    pub navigation_map_rid: Option<Rid>,
}

#[allow(clippy::derivable_impls, clippy::enum_variant_names)]
#[enum_dispatch]
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[derive(EnumDiscriminants)]
#[strum_discriminants(name(ActionType))]
pub enum Action {
    GoTo,
    Patrol,
    Animate,
    ArmWeapon,
    AimWeapon,
    RangedAttack,
    ReleaseWeapon,
}


// #[allow(clippy::enum_variant_names)]
// #[derive(Debug, Clone, ActionDispatchEnum, Serialize, Deserialize, Hash, PartialEq, Eq)]
// pub enum Action {
//     // ActivateObjectAction(ActionComponent,
//     #[implementation = "goap_actions::animate_action"]
//     Animate(ActionComponent),
//     // AttackGrenadeAction(ActionComponent),
//     // AttackLungeJumpAction(ActionComponent),
//     // AttackMeleeAction(ActionComponent),
//     #[implementation = "goap_actions::attack_ranged_action"]
//     AttackRanged(ActionComponent),
//     #[implementation = "goap_actions::aim_action"]
//     Aim(ActionComponent),
//     #[implementation = "goap_actions::deploy_weapon_action"]
//     DeployWeapon(ActionComponent),
//     #[implementation = "goap_actions::release_weapon_action"]
//     ReleaseWeapon(ActionComponent),
//     // HolsterWeapon(ActionComponent),
//     // BurnAction(ActionComponent),
//     // DeathAction(ActionComponent),
//     // DodgeBackAction(ActionComponent),
//     // DodgeRollToCoverAction(ActionComponent),
//     // #[implementation = "goap_actions::draw_weapon_action"]
//     // DrawWeapon(ActionComponent),
//     // FaceNodeAction(ActionComponent),
//     // FlushEnemyWithGrenadeAction(ActionComponent),
//     #[implementation = "goap_actions::goto_action"]
//     GoTo(ActionComponent),
//     // IdleAction(ActionComponent),
//     // LaySuppressiveFireAction(ActionComponent),
//     #[implementation = "goap_actions::patrol_action"]
//     Patrol(ActionComponent),
//     // ReactToDangerAction(ActionComponent),
//     #[implementation = "goap_actions::recover_from_attack_action"]
//     RecoverFromAttack(ActionComponent),
//     // ReloadAction(ActionComponent),
//     // ReloadInCoverAction(ActionComponent),
//     // RushEnemyAction(ActionComponent),
//     // StaggerAction(ActionComponent),
//     // StunnedAction(ActionComponent),
//     // SurprisedAction(ActionComponent),
// }

// impl PlanAction<AgentActionPlanContext<'_>> for Action {
//     fn get_action_preconditions(&self) -> &WorldState {
//         self.get_preconditions()
//     }
//
//     fn check_action_procedural_preconditions(
//         &self,
//         action_arguments: &AgentActionPlanContext,
//     ) -> bool {
//         self.check_procedural_preconditions(action_arguments)
//     }
//
//     fn get_action_effects<'a, 'b: 'a>(
//         &'a self,
//         action_arguments: &'b AgentActionPlanContext,
//     ) -> &'a WorldState {
//         self.get_effects(action_arguments)
//     }
//
//     fn get_action_cost(&self, action_arguments: &AgentActionPlanContext) -> u32 {
//         self.get_cost(action_arguments)
//     }
// }

#[enum_dispatch(Action)]
pub trait ActionBehavior {

    /// called when action is being activated
    fn execute_action(&self, inner: &ActionComponent, action_arguments: AgentActionWorldContext);

    /// called when action is being deactivated
    fn finish(&self, action_arguments: AgentActionWorldContext);

    /// returns true if action has been completed, false otherwise
    fn is_action_complete(&self, action_arguments: &AgentActionWorldContext) -> bool;

    /// returns true if action can be interrupted
    #[allow(unused_variables)]
    fn is_action_interruptible(&self, action_arguments: &AgentActionWorldContext) -> bool {true}

    #[allow(unused_variables)]
    fn check_procedural_preconditions(&self, action_arguments: &AgentActionPlanContext) -> bool {true}

    #[allow(unused_variables)]
    fn get_cost(&self, action_arguments: &AgentActionPlanContext) -> u32 {0}
}
