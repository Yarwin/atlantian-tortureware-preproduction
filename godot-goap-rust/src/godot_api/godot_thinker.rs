use crate::godot_api::ai_manager::GodotAIManager;
use godot::classes::{
    AnimationTree, Marker3D, NavigationAgent3D, Shape3D,
};
use godot::prelude::*;
use crate::ai::working_memory::Event::AnimationCompleted;
use crate::ai::working_memory::WMProperty;
use crate::character_controler::character_controller_3d::CharacterController3D;
use crate::godot_api::{CONNECT_DEFERRED, CONNECT_ONE_SHOT};
use crate::godot_api::gamesys::{GameSys, GameSystem};

/// an interface to speak with AI manager
#[derive(GodotClass, Debug)]
#[class(init, base=Node3D, rename=Thinker)]
pub struct GodotThinker {
    #[var]
    #[init(default = true)]
    pub is_active: bool,
    /// thinker id, used for load/save
    #[export]
    pub thinker_id: u32,
    /// dumb hack before proper groups will be implemented
    #[var(usage_flags = [GROUP, EDITOR, READ_ONLY])]
    init_files: u32,
    #[export(file = "*.ron")]
    pub(crate) actions_file: GString,
    #[export(file = "*.ron")]
    pub(crate) goals_file: GString,
    #[export(file = "*.ron")]
    pub(crate) sensors_file: GString,
    #[export(file = "*.ron")]
    pub(crate) animation_data: GString,
    #[export(file = "*.ron")]
    pub(crate) initial_state: GString,
    #[var(usage_flags = [GROUP, EDITOR, READ_ONLY])]
    references: u32,
    /// Area3D used to find nearby AI Nodes
    #[export]
    pub navigation_agent: Option<Gd<NavigationAgent3D>>,
    #[export]
    pub character_body: Option<Gd<CharacterController3D>>,
    #[export]
    pub animation_tree: Option<Gd<AnimationTree>>,
    #[export]
    pub head_position: Option<Gd<Marker3D>>,
    #[var(usage_flags = [GROUP, EDITOR, READ_ONLY])]
    detection_shapes: u32,
    /// detection shapes
    #[export]
    pub ainodes_detection_shape: Option<Gd<Shape3D>>,
    #[export]
    pub vision_detection_shape: Option<Gd<Shape3D>>,
    #[export]
    pub base_vision_detection_strength: f32,
    #[export]
    pub avoidance_detection_radius: f32,
    #[export]
    pub agent_radius: f32,
    #[var(usage_flags = [GROUP, EDITOR, READ_ONLY])]
    rotation: u32,
    #[export]
    pub rotation_speed_walk: f32,
    #[export]
    pub rotation_speed_normal: f32,
    #[export]
    pub rotation_speed_fast: f32,
    #[var(usage_flags = [GROUP, EDITOR, READ_ONLY])]
    movement: u32,
    #[export]
    pub movement_speed_multiplier: f32,
    #[export]
    pub walk_speed_mod: f32,
    #[export]
    pub dash_speed_mod: f32,
    base: Base<Node3D>,
}

impl GodotThinker {}

#[godot_api]
impl GodotThinker {
    #[func]
    fn register_self(&mut self) {
        let mut ai_manager = GodotAIManager::singleton();
        self.thinker_id = ai_manager.bind_mut().register_thinker(self);
    }

    #[func]
    fn on_animation_finished(&self, animation_name: StringName) {
        if self.thinker_id == 0 {return;}
        let fact = WMProperty::Event(AnimationCompleted(animation_name.into()));
        let mut ai_manager = GodotAIManager::singleton();
        ai_manager.bind_mut().add_new_wm_fact(self.thinker_id, fact, 1.0, 3.0);
    }

    #[func]
    fn get_target(&self) -> Variant {
        if self.thinker_id == 0 {return Variant::nil();}
        let ai_manager = GodotAIManager::singleton();
        let t = ai_manager.bind().get_thinker_target(self.thinker_id);
        t
    }
}

#[godot_api]
impl INode3D for GodotThinker {
    fn ready(&mut self) {
        if GameSys::singleton().bind().is_initialized {
            self.base_mut().call_deferred("register_self".into(), &[]);
        } else {
            let callable = self.base().callable("register_self");
            GameSys::singleton().connect_ex("initialization_completed".into(), callable).flags(CONNECT_ONE_SHOT + CONNECT_DEFERRED).done();
        }
    }
}
