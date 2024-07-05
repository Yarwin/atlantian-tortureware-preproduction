use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::ai::working_memory::{FactQuery, FactQueryCheck, NodeType, WorkingMemoryFactType, WorkingMemoryFactValueNodeTypeKey};
use crate::ai_nodes::godot_ai_node::{AINodeType};
use crate::sensors::sensor_types::SensorArguments;
use crate::sensors::sensor_types::SensorPolling;
use godot::prelude::*;
use crate::ai_nodes::ai_node::AINode;



/// sensor responsible for finding nearest valid patrol point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatrolPointSensor {
    update_every: f64,
    last_update_delta: f64
}


impl PatrolPointSensor {
    const MINIMAL_DIST: f32 = 1.6;

    fn find_nearest(args: &mut SensorArguments) -> Option<(Arc<Mutex<AINode>>, Vector3)> {
        let ainodes = args.polls.get_ainodes()?;

        let thinker_position = args.blackboard.thinker_position;
        let (mut best_distance, mut best_node) = (0.0, None);
        for (node_id, node_type) in ainodes {
            if *node_type != AINodeType::Patrol {
                continue
            }
            let Ok(reader) = args.ainodes.read() else {panic!("reader poisoned!")};
            let node = reader.get(node_id).expect("logic error – no node with given id").clone();
            let Ok(node_guard) = node.lock() else {panic!("mutex poisoned!")};

            if let AINode::Patrol {base, next, ..} = &*node_guard {
                let distance = base.position.distance_to(thinker_position);
                // agent is standing on the patrol node. Take its dependency (next node)
                if distance < Self::MINIMAL_DIST && next.is_some() {
                    let Ok(next_node_guard) = reader.get(&next.unwrap()).expect("logic error – no node with given id").lock() else {panic!("mutex poisoned!")};
                    // bail if next node is locked.
                    if !next_node_guard.is_locked() {
                        if let AINode::Patrol {base: b, ..} = &*next_node_guard {
                            return Some((reader.get(&next.unwrap()).unwrap().clone(), b.position))
                        }
                    }
                }

                // bail if current node is locked
                if node_guard.is_locked() {
                    continue
                }

                if (distance < best_distance || best_node.is_none()) && distance > Self::MINIMAL_DIST {
                    best_node = Some((reader.get(node_id).unwrap().clone(), base.position));
                    best_distance = distance;
                }
            };
        }
        best_node
    }
}

impl SensorPolling for PatrolPointSensor {
    fn process(&mut self, delta: f64, args: &mut SensorArguments) -> bool {
        self.last_update_delta += delta;
        if self.last_update_delta < self.update_every {
            return false;
        }
        self.last_update_delta = 0.0;
        if args.polls.get_ainodes().is_none() {return false}

        let fact_query = FactQuery::with_check(FactQueryCheck::NodeValue(WorkingMemoryFactValueNodeTypeKey::Patrol));
        let is_patrol_point_already_known = args.working_memory.find_fact(fact_query).is_some();
        if is_patrol_point_already_known {
            return false
        }

        let nearest_node = Self::find_nearest(args);
        if let Some((node, pos)) = nearest_node {
            args.working_memory.add_working_memory_fact(WorkingMemoryFactType::Node(NodeType::Patrol { ainode: node, position: pos }), 1.0, 200.0);
        }

        false
    }
}
