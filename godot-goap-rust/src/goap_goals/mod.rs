pub mod basic_goal;
mod dodge_goal;
pub mod goal_component;
pub mod goal_types;
mod kill_enemy_goal;
mod patrol_goal;
mod surprised_by_enemy_goal;
mod chase_enemy_goal;

// rust doesn't allow partial borrows in the Context of the struct – therefore we are creating the proper view using this macro.
#[macro_export]
macro_rules! thinker_process_to_goal_view {
    ($thinker: ident) => {{
        $crate::goap_goals::goal_types::AgentGoalWorldContext {
            id: &$thinker.id,
            working_memory: $thinker.working_memory,
            current_world_state: $thinker.world_state,
            blackboard: $thinker.blackboard,
            ai_nodes: $thinker.ai_nodes,
        }
    }};
}