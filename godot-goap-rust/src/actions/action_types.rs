use std::hash::{Hash};
use std::sync::{Arc};
use godot::builtin::Rid;
use serde::{Deserialize, Serialize};
use static_enum_dispatch::{ActionDispatchEnum};
use crate::actions;
use crate::actions::action_component::ActionComponent;
use crate::ai::blackboard::Blackboard;
use crate::ai::planner::PlanAction;
use crate::ai::working_memory::WorkingMemory;
use crate::ai::world_state::WorldState;
use crate::animations::animation_data::AnimationsData;


#[derive(Debug)]
pub struct AgentActionWorldContext<'a> {
    pub working_memory: &'a WorkingMemory,
    pub current_world_state: &'a mut WorldState,
    pub blackboard: &'a mut Blackboard,
    pub navigation_map_rid: Option<Rid>,
    pub animations: &'a Arc<AnimationsData>
}


#[derive(Debug)]
pub struct AgentActionPlanContext<'a> {
    pub working_memory: &'a mut WorkingMemory,
    pub current_world_state: &'a mut WorldState,
    pub blackboard: &'a mut Blackboard,
    pub navigation_map_rid: Option<Rid>
}


#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, ActionDispatchEnum, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum Action {
    // ActivateObjectAction(ActionComponent,
    #[implementation="actions::animate_action"]
    Animate(ActionComponent),
    // AttackGrenadeAction(ActionComponent),
    // AttackLungeJumpAction(ActionComponent),
    // AttackMeleeAction(ActionComponent),
    #[implementation="actions::attack_ranged_action"]
    AttackRangedAction(ActionComponent),
    // BurnAction(ActionComponent),
    // DeathAction(ActionComponent),
    // DodgeBackAction(ActionComponent),
    // DodgeRollToCoverAction(ActionComponent),
    #[implementation="actions::draw_weapon_action"]
    DrawWeaponAction(ActionComponent),
    // FaceNodeAction(ActionComponent),
    // FlushEnemyWithGrenadeAction(ActionComponent),
    #[implementation="actions::goto_action"]
    GoTo(ActionComponent),
    // IdleAction(ActionComponent),
    // LaySuppressiveFireAction(ActionComponent),
    #[implementation="actions::patrol_action"]
    Patrol(ActionComponent),
    // ReactToDangerAction(ActionComponent),
    #[implementation="actions::recover_from_attack_action"]
    RecoverFromAttackAction(ActionComponent),
    // ReloadAction(ActionComponent),
    // ReloadInCoverAction(ActionComponent),
    // RushEnemyAction(ActionComponent),
    // StaggerAction(ActionComponent),
    // StunnedAction(ActionComponent),
    // SurprisedAction(ActionComponent),
}


impl PlanAction<AgentActionPlanContext<'_>> for Action {
    fn get_action_preconditions(&self) -> &WorldState {
        self.get_preconditions()
    }

    fn check_action_procedural_preconditions(&self, action_arguments: &AgentActionPlanContext) -> bool {
        self.check_procedural_preconditions(action_arguments)
    }

    fn get_action_effects<'a, 'b: 'a>(&'a self, action_arguments: &'b AgentActionPlanContext) -> &'a WorldState {
        self.get_effects(action_arguments)
    }

    fn get_action_cost(&self, action_arguments: &AgentActionPlanContext) -> u32 {
        self.get_cost(action_arguments)
    }
}

pub trait ActionBehavior {
    fn get_effects<'a, 'b: 'a>(&'a self, action_arguments: &'b AgentActionPlanContext) -> &'a WorldState;
    fn get_preconditions(&self) -> &WorldState;
    /// called when action is being activated
    fn execute_action(&self, action_arguments: AgentActionWorldContext);
    /// called when action is being deactivated
    fn finish(&self, action_arguments: AgentActionWorldContext);
    /// returns true if action has been completed, false otherwise
    fn is_action_complete(&self, action_arguments: &AgentActionWorldContext) -> bool;
    /// returns true if action can be interrupted
    fn is_action_interruptible(&self, action_arguments: &AgentActionWorldContext) -> bool;
    /// check action's procedural preconditions
    fn check_procedural_preconditions(&self, action_arguments: &AgentActionPlanContext) -> bool;
    fn get_cost(&self, action_arguments: &AgentActionPlanContext) -> u32;
}
