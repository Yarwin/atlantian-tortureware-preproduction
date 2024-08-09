use godot::classes::IMainLoop;
use godot::prelude::*;
use crate::act_react::act_react_executor::ActReactExecutor;
use crate::godot_api::ai_manager::GodotAIManager;
use crate::godot_api::gamesys::GameSystem;
use crate::godot_api::inventory_manager::InventoryManager;

#[derive(GodotClass)]
#[class(init, base=SceneTree)]
struct AtlantianTorturewareGameLoop {
    inventory_manager: Option<Gd<InventoryManager>>,
    ai_manager: Option<Gd<GodotAIManager>>,
    act_react_executor: Option<Gd<ActReactExecutor>>,
}

impl AtlantianTorturewareGameLoop {}


#[godot_api]
impl ISceneTree for AtlantianTorturewareGameLoop {
    fn initialize(&mut self) {
        self.inventory_manager = Some(InventoryManager::initialize());
        self.ai_manager = Some(GodotAIManager::initialize());
        self.act_react_executor = Some(ActReactExecutor::initialize());
    }

    fn physics_process(&mut self, delta: f64) -> bool {
        self.ai_manager.as_mut().unwrap().bind_mut().physics_process(delta);
        self.act_react_executor.as_mut().unwrap().bind_mut().physics_process(delta);
        false
    }

    fn finalize(&mut self) {
        self.ai_manager.as_mut().unwrap().bind_mut().exit();
        self.ai_manager.as_mut().unwrap().call_deferred("free".into(), &[]);
        self.inventory_manager.as_mut().unwrap().bind_mut().exit();
        self.inventory_manager.as_mut().unwrap().call_deferred("free".into(), &[]);
        self.act_react_executor.as_mut().unwrap().bind_mut().exit();
        self.act_react_executor.as_mut().unwrap().call_deferred("free".into(), &[]);
    }
}