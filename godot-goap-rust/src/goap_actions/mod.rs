pub mod action_component;
pub mod action_types;
mod animate_action;
mod attack_ranged_action;
mod base_action;
mod draw_weapon_action;
mod goto_action;
mod patrol_action;
mod recover_from_attack_action;
mod aim_action;
mod deploy_weapon_action;
mod release_weapon_action;

// rust doesn't allow partial borrows in the Context of the struct – therefore we are creating the proper view using this macro.
#[macro_export]
macro_rules! action_arguments {
    ($thinker: ident) => {{
        $crate::goap_actions::action_types::AgentActionWorldContext {
            working_memory: $thinker.working_memory,
            current_world_state: $thinker.world_state,
            blackboard: $thinker.blackboard,
            navigation_map_rid: $thinker.navigation_map_rid.clone(),
            animations: &$thinker.animations,
            ai_nodes: $thinker.ai_nodes
        }
    }};
}

#[macro_export]
macro_rules! action_plan_context {
    ($thinker: ident) => {{
        $crate::goap_actions::action_types::AgentActionPlanContext {
            working_memory: $thinker.working_memory,
            current_world_state: $thinker.world_state,
            blackboard: $thinker.blackboard,
            navigation_map_rid: $thinker.navigation_map_rid.clone(),
        }
    }};
}
