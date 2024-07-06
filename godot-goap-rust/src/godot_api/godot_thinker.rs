use crate::godot_api::ai_manager::GodotAIManager;
use godot::classes::{
    AnimationTree, CharacterBody3D, Engine, Marker3D, NavigationAgent3D, Shape3D,
};
use godot::prelude::*;

/// an interface to speak with AI manager
#[derive(GodotClass, Debug)]
#[class(init, base=Node3D, rename=thinker)]
pub struct GodotThinker {
    #[export(file = "*.ron")]
    pub(crate) actions_file: GString,
    #[export(file = "*.ron")]
    pub(crate) goals_file: GString,
    #[export(file = "*.ron")]
    pub(crate) sensors_file: GString,
    #[export(file = "*.ron")]
    pub(crate) animation_data: GString,
    /// Area3D used to find nearby AI Nodes
    #[export]
    pub navigation_agent: Option<Gd<NavigationAgent3D>>,
    #[export]
    pub character_body: Option<Gd<CharacterBody3D>>,
    #[export]
    pub animation_tree: Option<Gd<AnimationTree>>,
    #[export]
    pub head_position: Option<Gd<Marker3D>>,
    /// detection shapes
    #[export]
    pub ainodes_detection_shape: Option<Gd<Shape3D>>,
    #[export]
    pub vision_detection_shape: Option<Gd<Shape3D>>,
    #[export]
    pub base_vision_detection_strenght: f32,
    #[export]
    pub avoidance_detection_radius: f32,
    #[export]
    pub agent_radius: f32,
    #[var]
    #[init(default = true)]
    pub is_active: bool,
    #[export]
    pub rotation_speed: f32,
    #[export]
    pub acceleration: f32,
    #[export]
    pub movement_speed: f32,
    #[export]
    pub walk_speed_mod: f32,
    #[export]
    pub dash_speed_mod: f32,

    /// thinker id, used for load/save
    #[export]
    pub thinker_id: u32,
    base: Base<Node3D>,
}

impl GodotThinker {}

#[godot_api]
impl GodotThinker {
    #[func]
    fn register_self(&mut self) {
        let mut ai_manager = Engine::singleton()
            .get_singleton("AIManager".into())
            .unwrap()
            .cast::<GodotAIManager>();
        self.thinker_id = ai_manager.bind_mut().register_thinker(self);
    }
}

#[godot_api]
impl INode3D for GodotThinker {
    fn ready(&mut self) {
        self.base_mut().call_deferred("register_self".into(), &[]);
    }
}
