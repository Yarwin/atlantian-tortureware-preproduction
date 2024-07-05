use crate::goals::goal_types::{AgentGoalWorldContext, GoalBehaviour};
use serde::{Serialize, Deserialize};
use crate::ai::blackboard::MovementSpeed;
use crate::ai::working_memory::{FactQuery, FactQueryCheck, NodeType, WorkingMemoryFactType, WorkingMemoryFactValueNodeTypeKey};
use crate::ai_nodes::ai_node::{AINode, AINodeStatus};
use crate::ai_nodes::ai_node::AINodeStatus::Locked;
use crate::goals::goal_component::GoalComponent;
use crate::targeting::target::Target;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PatrolGoal;

impl GoalBehaviour for PatrolGoal {
    fn is_valid(&self, _goal: &GoalComponent, agent_world_context: &AgentGoalWorldContext) -> bool {
        let fact_query = FactQuery::with_check(FactQueryCheck::NodeValue(WorkingMemoryFactValueNodeTypeKey::Patrol));
            if agent_world_context.working_memory.find_fact(fact_query).is_some() {
                return true
            }

        false
    }

    /// save patrol point target in the blackboard.
    fn activate(&self, _goal: &GoalComponent, agent_world_context: &mut AgentGoalWorldContext) -> bool {
        let fact_query = FactQuery::with_check(FactQueryCheck::NodeValue(WorkingMemoryFactValueNodeTypeKey::Patrol));
        if let Some(fact) = agent_world_context.working_memory.find_fact_mut(fact_query) {
            // check & lock given patrol node
            if let WorkingMemoryFactType::Node(NodeType::Patrol {ainode, position}) = &mut fact.f_type {
                let Ok(mut ainode_guard) = ainode.lock() else { panic!("mutex failed!") };
                if ainode_guard.is_locked() {
                    return false;
                }
                let AINode::Patrol { base, ..} = &mut *ainode_guard else { panic!("no such node!") };
                base.status = Locked(*agent_world_context.id);
                agent_world_context.blackboard.target = Some(Target::PatrolPoint(ainode.clone(), *position));
                agent_world_context.blackboard.walk_speed = MovementSpeed::Walk;
                return true
            }
        }
        false
    }

    /// remove patrol point target
    fn deactivate(&self, _goal: &GoalComponent, agent_world_context: &mut AgentGoalWorldContext) {
        let fact_query = FactQuery::with_check(FactQueryCheck::NodeValue(WorkingMemoryFactValueNodeTypeKey::Patrol));
        // unlock patrol node & remove it from memory
        if let Some(fact) = agent_world_context.working_memory.find_and_mark_as_invalid(fact_query) {
            let WorkingMemoryFactType::Node(NodeType::Patrol { ainode, .. }) = &fact.f_type else { return; };
            let Ok(mut ainode_guard) = ainode.lock() else { panic!("mutex poisoned!") };
            ainode_guard.base_mut().status = AINodeStatus::Free;
        }
    }
}
