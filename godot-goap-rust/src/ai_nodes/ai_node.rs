use crate::ai_nodes::godot_ai_node::{AINodeType, GodotAINode};
use atomic::{AtomicU32, Ordering};
use godot::prelude::*;
use std::sync::atomic;

/// an abstraction that allows level designer to specify various points of interest for an AI
#[derive(Debug, Default)]
pub enum AINode {
    #[default]
    None,
    Patrol {
        base: AINodeBase,
        next: Option<u32>,
        orientation: Option<Vector3>,
    },
}

impl AINode {
    pub fn base(&self) -> &AINodeBase {
        match self {
            AINode::Patrol { base, .. } => base,
            _ => {
                todo!()
            }
        }
    }
    pub fn base_mut(&mut self) -> &mut AINodeBase {
        match self {
            AINode::Patrol { base, .. } => base,
            _ => {
                todo!()
            }
        }
    }
    pub fn with_dependency(self, dependency: u32) -> Self {
        match self {
            AINode::Patrol {
                base,
                next: _,
                orientation,
            } => AINode::Patrol {
                base,
                next: Some(dependency),
                orientation,
            },
            _ => {
                godot_print!("what");
                unimplemented!()
            }
        }
    }

    pub fn is_locked_not_by(&self, not_by: u32) -> bool {
        match self {
            AINode::Patrol { base, .. } => {
                let val = base.status.load(Ordering::Acquire);
                !((val != not_by) || (val == 0))
            }
            _ => todo!(),
        }
    }

    pub fn is_locked(&self) -> bool {
        match self {
            AINode::Patrol { base, .. } => base.is_locked(),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Default)]
pub enum AINodeStatus {
    #[default]
    Free,
    /// locked by an agent with given id
    Locked(u32),
}

impl From<&GodotAINode> for AINode {
    fn from(value: &GodotAINode) -> Self {
        let dependency = value.dependency.as_ref().map(|d| d.bind().ainode_id);
        let inner = AINodeBase {
            ainode_id: value.ainode_id,
            base_id: value.base().instance_id(),
            position: value.base().get_global_position(),
            status: AtomicU32::new(0),
        };
        match value.node_type {
            AINodeType::Invalid => {
                todo!()
            }
            AINodeType::Patrol => AINode::Patrol {
                base: inner,
                next: dependency,
                orientation: value
                    .orientation_node
                    .as_ref()
                    .map(|on| on.get_global_position()),
            },
            AINodeType::Hide => {
                todo!()
            }
            AINodeType::Ambush => {
                todo!()
            }
            AINodeType::Cover => {
                todo!()
            }
        }
    }
}

#[derive(Debug)]
pub struct AINodeBase {
    pub ainode_id: u32,
    pub base_id: InstanceId,
    pub position: Vector3,
    /// status: stores 0 when free, agent_id otherwise
    pub status: AtomicU32,
}

impl AINodeBase {
    pub fn is_locked(&self) -> bool {
        self.status.load(Ordering::Acquire) != 0
    }
}
