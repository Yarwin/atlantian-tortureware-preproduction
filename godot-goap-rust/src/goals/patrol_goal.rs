use godot::global::godot_print;
use crate::ai::blackboard::{MovementSpeed, NavigationTarget};
use crate::ai::working_memory::{FactQuery, FactQueryCheck, Node, WMNodeType, WorkingMemoryFactType};
use crate::ai_nodes::ai_node::AINodeStatus::Locked;
use crate::ai_nodes::ai_node::{AINode, AINodeStatus};
use crate::goals::goal_component::GoalComponent;
use crate::goals::goal_types::{AgentGoalWorldContext, GoalBehaviour};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PatrolGoal;

impl GoalBehaviour for PatrolGoal {
    fn is_valid(&self, _goal: &GoalComponent, agent_world_context: &AgentGoalWorldContext) -> bool {
        if agent_world_context.blackboard.current_locked_node.is_some() {
            return true
        }
        let fact_query = FactQuery::with_check(FactQueryCheck::Node(
            WMNodeType::Patrol,
        ));
        if agent_world_context
            .working_memory
            .find_fact(fact_query)
            .is_some()
        {
            return true;
        }
        false
    }

    /// save patrol point target in the blackboard.
    fn activate(
        &self,
        _goal: &GoalComponent,
        agent_world_context: &mut AgentGoalWorldContext,
    ) -> bool {
        godot_print!("activating patrol goal…");
        let fact_query = FactQuery::with_check(FactQueryCheck::Node(
            WMNodeType::Patrol,
        ));
        if let Some(fact) = agent_world_context.working_memory.find_fact_mut(fact_query) {
            // check & lock given patrol node
            if let WorkingMemoryFactType::Node(Node::Patrol { ainode_id, position }) =
                &mut fact.f_type
            {
                let mut ainodes_guard = agent_world_context.ai_nodes.as_mut().unwrap().lock().expect("mutex failed!");
                let ainode = ainodes_guard.get_mut(ainode_id).expect("no node with given id!");
                if ainode.is_locked() {
                    let fact_query = FactQuery::with_check(FactQueryCheck::Node(
                        WMNodeType::Patrol,
                    ));
                    agent_world_context.working_memory.mark_as_invalid(fact_query);
                    return false;
                }
                let AINode::Patrol { base, .. } = &mut *ainode else {
                    panic!("no such node!")
                };
                base.status = Locked(*agent_world_context.id);
                agent_world_context.blackboard.current_locked_node = Some(*ainode_id);
                agent_world_context.blackboard.navigation_target =
                    Some(NavigationTarget::PatrolPoint(*ainode_id, *position));
                agent_world_context.blackboard.walk_speed = MovementSpeed::Walk;
                return true;
            }
        }
        false
    }

    /// remove patrol point target
    fn deactivate(&self, _goal: &GoalComponent, agent_world_context: &mut AgentGoalWorldContext) {
        let fact_query = FactQuery::with_check(FactQueryCheck::Node(
            WMNodeType::Patrol,
        ));
        agent_world_context
            .working_memory
            .mark_as_invalid(fact_query);
        if let Some(ainode_id) = agent_world_context.blackboard.current_locked_node.take() {
            godot_print!("freeing node!");
            let Ok(mut ainodes_guard) = agent_world_context.ai_nodes.as_mut().expect("no ainodes").lock() else {panic!("mutex failed!")};
            let ainode = ainodes_guard.get_mut(&ainode_id).expect("no ainode with such id!");
            ainode.base_mut().status = AINodeStatus::Free;
        }
    }
}
