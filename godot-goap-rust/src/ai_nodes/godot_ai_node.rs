use crate::ai_nodes::ai_node::AINodeStatus;
use crate::godot_api::ai_manager::GodotAIManager;
use godot::classes::{Area3D, Engine, IArea3D, Marker3D};
use godot::prelude::*;

#[derive(GodotConvert, Var, Export, Clone, Debug, Copy, Default, PartialEq, Eq)]
#[godot(via = u32)]
pub enum AINodeType {
    #[default]
    Invalid,
    Patrol,
    Hide,
    Ambush,
    Cover,
}

#[derive(GodotClass)]
#[class(init, base=Area3D, rename=AINode)]
pub struct GodotAINode {
    #[export]
    #[init(default = 0)]
    pub ainode_id: u32,
    #[export]
    pub node_type: AINodeType,
    /// linked AI nodes that follows this AI node
    #[export]
    pub dependency: Option<Gd<GodotAINode>>,
    #[export]
    pub orientation_node: Option<Gd<Marker3D>>,
    #[export]
    pub animatable_object: Option<Gd<Node3D>>,
    pub ainode_status: AINodeStatus,
    pub base: Base<Area3D>,
}

#[godot_api]
impl IArea3D for GodotAINode {
    fn ready(&mut self) {
        let mut ai_manager = Engine::singleton()
            .get_singleton("AIManager".into())
            .unwrap()
            .cast::<GodotAIManager>();
        self.ainode_id = ai_manager.bind_mut().register_ainode(self);
    }
}

#[godot_api]
impl GodotAINode {}
