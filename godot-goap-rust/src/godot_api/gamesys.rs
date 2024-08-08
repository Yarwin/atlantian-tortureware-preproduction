use godot::classes::Engine;
use godot::obj::{bounds, Bounds, NewAlloc};
use godot::prelude::*;
use crate::act_react::act_react_executor::ActReactExecutor;
use crate::godot_api::ai_manager::GodotAIManager;
use crate::godot_api::inventory_manager::InventoryManager;

/// A node responsible for initializing&keeping all the game systems and acting as an event bus.
#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct GameSys {
    #[var]
    #[init(default = OnReady::manual())]
    pub inventory_manager: OnReady<Gd<InventoryManager>>,
    #[var]
    #[init(default = OnReady::manual())]
    pub ai_manager: OnReady<Gd<GodotAIManager>>,
    #[var]
    #[init(default = OnReady::manual())]
    pub act_react_executor: OnReady<Gd<ActReactExecutor>>,
    #[var]
    pub is_initialized: bool,
    base: Base<Node>,
}

#[godot_api]
impl INode for GameSys {
    fn process(&mut self, _delta: f64) {}

    fn physics_process(&mut self, delta: f64) {
        self.ai_manager.bind_mut().physics_process(delta);
        self.act_react_executor.bind_mut().physics_process(delta);
    }

    fn enter_tree(&mut self) {
        Engine::singleton().register_singleton(Self::singleton_name(), self.base().clone().upcast::<Object>());
    }

    fn exit_tree(&mut self) {
        self.ai_manager.bind_mut().exit();
        self.ai_manager.call_deferred("free".into(), &[]);
        self.inventory_manager.bind_mut().exit();
        self.inventory_manager.call_deferred("free".into(), &[]);
        self.act_react_executor.bind_mut().exit();
        self.act_react_executor.call_deferred("free".into(), &[]);
        Engine::singleton().unregister_singleton(Self::singleton_name());
    }

    fn ready(&mut self) {
        self.inventory_manager.init(InventoryManager::initialize());
        self.ai_manager.init(GodotAIManager::initialize());
        self.act_react_executor.init(ActReactExecutor::initialize());
        self.is_initialized = true;
    }
}

#[godot_api]
impl GameSys {
    /// emitted when all the systems are initialized
    #[signal]
    fn initialization_completed();

    #[signal]
    fn hud_visibility_changed(hidden: bool);

}

impl GameSys {
    pub fn singleton() -> Gd<Self> {
        Engine::singleton()
            .get_singleton(Self::singleton_name())
            .unwrap()
            .cast::<GameSys>()
    }

    fn singleton_name() -> StringName {
        StringName::from("GameSystems")
    }
}

pub trait GameSystem: GodotClass + Bounds<Declarer = bounds::DeclUser> + NewAlloc + Inherits<Object> {
    fn singleton() -> Gd<Self> {
        Engine::singleton()
            .get_singleton(Self::singleton_name())
            .unwrap()
            .cast::<Self>()
    }
    fn singleton_name() -> StringName;
    fn initialize() -> Gd<Self> {
        let game_system = Self::new_alloc();
        Engine::singleton()
            .register_singleton(Self::singleton_name(), game_system.clone());
        game_system
    }

    fn exit(&mut self) {Engine::singleton().unregister_singleton(Self::singleton_name());}
    #[allow(unused_variables)]
    fn physics_process(&mut self, delta: f64) {}
    #[allow(unused_variables)]
    fn process(&mut self, delta: f64) {}
}
