use std::sync::atomic::Ordering;
use crate::ai::blackboard::{SpeedMod, NavigationTarget};
use crate::ai::working_memory::{FactQuery, FactQueryCheck, Node, WMNodeType, WMProperty};
use crate::ai_nodes::ai_node::{AINode};
use crate::goap_goals::goal_component::GoalComponent;
use crate::goap_goals::goal_types::{AgentGoalWorldContext, GoalBehaviour};
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
        let fact_query = FactQuery::with_check(FactQueryCheck::Node(
            WMNodeType::Patrol,
        ));
        if let Some(fact) = agent_world_context.working_memory.find_fact_mut(fact_query) {
            // check & lock given patrol node
            if let WMProperty::Node(Node::Patrol { ainode_id, position }) =
                &mut fact.f_type
            {
                let ainodes_guard = agent_world_context.ai_nodes.as_mut().unwrap().read().expect("rwlock failed!");
                let ainode = ainodes_guard.get(ainode_id).expect("no node with given id!");
                if ainode.is_locked() {
                    let fact_query = FactQuery::with_check(FactQueryCheck::Node(
                        WMNodeType::Patrol,
                    ));
                    agent_world_context.working_memory.mark_as_invalid(fact_query);
                    return false;
                }
                let AINode::Patrol { base, .. } = ainode else {
                    panic!("no such node!")
                };
                base.status.store(*agent_world_context.id, Ordering::Release);
                agent_world_context.blackboard.current_locked_node = Some(*ainode_id);
                agent_world_context.blackboard.navigation_target =
                    Some(NavigationTarget::PatrolPoint(*ainode_id, *position));
                agent_world_context.blackboard.walk_speed = SpeedMod::Slow;
                agent_world_context.blackboard.rotation_speed = SpeedMod::Slow;
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
            let Ok(ainodes_guard) = agent_world_context.ai_nodes.as_mut().expect("no ainodes").read() else {panic!("mutex failed!")};
            let ainode = ainodes_guard.get(&ainode_id).expect("no ainode with such id!");
            ainode.base().status.store(0, Ordering::Release)
        }
    }
}
