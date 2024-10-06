use crate::ai::process_plan::{process_plan, ThinkerPlanEvent, ThinkerProcess};
use crate::ai::thinker::{Thinker, ThinkerShared};
use crate::ai_nodes::ai_node::AINode;
use crate::ai_nodes::godot_ai_node::GodotAINode;
use crate::animations::animation_data::{AnimationProps, AnimationsData, AnimationType};
use crate::goap_goals::goal_component::GoalComponent;
use crate::godot_api::godot_thinker::GodotThinker;
use crate::godot_api::CONNECT_ONE_SHOT;
use crate::sensors::sensor_types::PollingSensor;
use crate::thinker_states::process_thinker::process_thinker;
use godot::classes::{Engine, FileAccess};
use godot::classes::file_access::ModeFlags;
use godot::prelude::*;
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::sync::{mpsc, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::ai::working_memory::WMProperty;
use crate::goap_actions::action_component::ActionComponent;
use crate::godot_api::gamesys::GameSystem;
use crate::utils::generate_id::{assign_id, ToCreate};


#[derive(GodotClass)]
#[class(init, base=Object, rename=AIManager)]
pub struct GodotAIManager {
    pub actions: HashMap<GString, Arc<Vec<ActionComponent>>>,
    pub goals: HashMap<GString, Arc<Vec<GoalComponent>>>,
    pub animations: HashMap<GString, Arc<AnimationsData>>,
    sensors_blueprint: HashMap<GString, Vec<PollingSensor>>,
    pub ai_nodes: Arc<RwLock<HashMap<u32, AINode>>>,
    ainode_id_with_dependencies: VecDeque<(u32, Gd<GodotAINode>)>,

    pub thinkers: HashMap<u32, Thinker>,
    #[init(default = Vec::new())]
    thinkers_to_create: Vec<ToCreate<GodotThinker>>,
    pub is_initialized: bool,
    pub current_thinker_id: u32,
    pub current_node_id: u32,
    pub sender: Option<Sender<ThinkerPlanEvent>>,
    pub receiver: Option<Receiver<()>>,
    pub thread: Option<thread::JoinHandle<()>>,
    base: Base<Object>,
}

#[godot_api]
impl IObject for GodotAIManager {}


#[godot_api]
impl GodotAIManager {
    #[func]
    fn post_ready(&mut self) {
        // update all dependencies
        for (node_id, dependency) in self.ainode_id_with_dependencies.drain(..) {
            let dep_node_id = dependency.bind().ainode_id;
            let Ok(mut ainodes_guard) = self.ai_nodes.write() else {
                panic!("Mutex failed!");
            };
            let ainode = ainodes_guard.get_mut(&node_id).expect("no such ainode!");
            let new_ainode = std::mem::take(&mut *ainode);
            *ainode = new_ainode.with_dependency(dep_node_id);
        }
    }

    #[func]
    fn create_thinkers(&mut self) {
        let mut to_drain = std::mem::take(&mut self.thinkers_to_create);
        to_drain.sort();
        to_drain.reverse();
        for to_create in to_drain.drain(..) {
            self.create_thinker(to_create);
        }
        self.thinkers_to_create = to_drain;
        self.is_initialized = true;
    }

    #[func]
    pub fn get_thinker_target(&self, thinker_id: u32) -> Variant {
        let thinker = self.thinkers.get(&thinker_id).unwrap();
        let shared = thinker.shared.lock().unwrap();
        let Some(target) = shared.blackboard.target.as_ref().map(|t| t.get_target_pos().map(|pos| pos.to_variant())) else {return Variant::nil()};
        target.unwrap_or(Variant::nil())
    }

    #[func]
    fn get_thinker_debug_data(&self, thinker_id: u32) -> Dictionary {
        let thinker = self.thinkers.get(&thinker_id).unwrap();
        let shared = thinker.shared.lock().unwrap();
        let current_goal: String = if let Some(current) = shared.blackboard.current_goal {
            thinker.goals[current].name.clone()
        } else {
            "no goal".into()
        };
        let current_action: String = if let Some(current) = shared.blackboard.current_action() {
            format!("{:?}", thinker.actions[current])
        } else {
            "no action".into()
        };
        let current_world_state = format!("{:?}", shared.world_state);
        dict! {
            "current_world_state": current_world_state,
            "goal": current_goal,
            "action": current_action
        }
    }

    #[func]
    fn unregister_thinker(&mut self, id: u32) {
        self.thinkers.remove(&id);
    }

    #[func]
    fn unregister_ainode(&mut self, id: u32) {
        let Ok(mut ai_nodes) = self.ai_nodes.write() else {
            panic!("RwLock Writer failed!");
        };
        ai_nodes.remove(&id);
    }
}

impl GodotAIManager {
    pub fn register_thinker(&mut self, thinker: ToCreate<GodotThinker>) {
        self.thinkers_to_create.push(thinker);
        if !self.is_initialized {
            self.base_mut().call_deferred("create_thinkers".into(), &[]);
        }
    }

    pub fn add_new_wm_fact(&mut self, thinker_id: u32, fact: WMProperty, confidence: f32, expiration: f64) {
        let Ok(mut guard) = self.thinkers[&thinker_id].shared.lock() else {panic!("mutex failed - couldn't add new wm fact!")};
        guard.working_memory.add_working_memory_fact(fact, confidence, expiration);
        drop(guard)
    }

    pub fn invalidate_plan(&mut self, thinker_id: u32) {
        let Ok(mut guard) = self.thinkers[&thinker_id].shared.lock() else {panic!("mutex failed! Couldn't invalidate the plan")};
        guard.blackboard.invalidate_plan = true;
    }

    pub fn register_ainode(&mut self, ai_node: &mut GodotAINode) -> u32 {
        let id: u32 = assign_id(ai_node.ainode_id, &mut self.current_node_id);
        // unregister on exit
        let callable = Callable::from_object_method(&self.base().clone(), "unregister_ainode")
            .bindv(array![id.to_variant()]);
        let _ = ai_node
            .base_mut()
            .connect_ex("tree_exiting".into(), callable)
            .flags(CONNECT_ONE_SHOT)
            .done();
        // update dependencies in next cycle
        if ai_node.dependency.is_some() {
            self.ainode_id_with_dependencies
                .push_front((id, ai_node.dependency.as_ref().unwrap().clone()));
        }
        let node = AINode::from(&*ai_node);
        let Ok(mut ai_nodes) = self.ai_nodes.write() else {
            panic!("RWLock failed!");
        };
        ai_nodes.insert(id, node);
        id
    }

    fn create_thinker(&mut self, mut to_create: ToCreate<GodotThinker>) {
        let id = assign_id(to_create.id, &mut self.current_thinker_id);

        // unregister on exit
        let callable = Callable::from_object_method(&self.base(), "unregister_thinker")
            .bindv(array![id.to_variant()]);
        let _ = to_create
            .instance
            .connect_ex("tree_exiting".into(), callable)
            .flags(CONNECT_ONE_SHOT)
            .done();
        let navigation_map_rid = to_create
            .instance
            .bind_mut()
            .navigation_agent
            .as_ref()
            .map(|agent| agent.get_navigation_map());

        let mut shared = ThinkerShared {
            working_memory: Default::default(),
            blackboard: Default::default(),
            world_state: Self::load(&to_create.instance.bind().initial_state),
            target_mask: Default::default(),
        };
        shared.blackboard.thinker_position = to_create.instance.get_global_position();

        let thinker = Thinker {
            id,
            base: Some(to_create.instance.clone()),
            is_active: to_create.instance.bind().is_active,
            actions: self.get_actions(&to_create.instance.bind().actions_file).unwrap(),
            goals: self.get_goals(&to_create.instance.bind().goals_file).unwrap(),
            polling_sensors: self.get_sensors(&to_create.instance.bind().sensors_file).unwrap(),
            animations: self
                .get_animations_data(&to_create.instance.bind().animation_data)
                .unwrap(),
            shared: Arc::new(Mutex::new(shared)),
            navigation_map_rid,
            ..Default::default()
        };
        self.thinkers.insert(id, thinker);
        to_create.instance.bind_mut().thinker_id = id;
    }

    fn load<T: for<'a> Deserialize<'a>>(path: &GString) -> T {
        let file = FileAccess::open(path.clone(), ModeFlags::READ);
        file.as_ref()
            .expect("Couldn't read given component config!");
        ron::from_str::<T>(&String::from(file.unwrap().get_as_text()))
            .expect("Couldn't read given component config!")
    }

    /// loads & caches given set of components
    fn load_components<T, U: for<'a> Deserialize<'a> + Into<T>>(
        collection: &mut HashMap<GString, Arc<Vec<T>>>,
        path: &GString,
    ) -> Option<Arc<Vec<T>>> {
        if let Some(collection) = collection.get(path) {
            return Some(collection.clone());
        }
        let components: Arc<Vec<T>> = Arc::new(
            Self::load::<Vec<U>>(path)
                .into_iter()
                .map(Into::into)
                .collect(),
        );
        collection.insert(path.clone(), components.clone());
        Some(components)
    }

    pub fn get_actions(&mut self, path: &GString) -> Option<Arc<Vec<ActionComponent>>> {
        Self::load_components::<ActionComponent, ActionComponent>(&mut self.actions, path)
    }

    pub fn get_goals(&mut self, path: &GString) -> Option<Arc<Vec<GoalComponent>>> {
        Self::load_components::<GoalComponent, GoalComponent>(&mut self.goals, path)
    }

    pub fn get_animations_data(&mut self, path: &GString) -> Option<Arc<AnimationsData>> {
        if let Some(collection) = self.animations.get(path) {
            return Some(collection.clone());
        }
        let components: HashMap<AnimationType, AnimationProps> = Self::load::<HashMap<AnimationType, AnimationProps>>(path);
        let animations_data = Arc::new(AnimationsData::from(components));
        self.animations.insert(path.clone(), animations_data.clone());
        Some(animations_data)
    }

    fn get_sensors(&mut self, path: &GString) -> Option<Vec<PollingSensor>> {
        if let Some(collection) = self.sensors_blueprint.get(path) {
            return Some(collection.clone());
        }
        let components: Vec<PollingSensor> = Self::load(path);
        self.sensors_blueprint
            .insert(path.clone(), components.clone());
        Some(components)
    }
}

impl GameSystem for GodotAIManager {
    const NAME: &'static str = "AIManager";
    fn initialize() -> Gd<Self> {
        let (process_sender, process_receiver) = mpsc::channel();
        let (update_sender, update_receiver) = mpsc::channel();

        let mut ai_manager = Self::new_alloc();
        ai_manager.bind_mut().sender = Some(process_sender);
        ai_manager.bind_mut().receiver = Some(update_receiver);
        ai_manager.bind_mut().thread = Some(thread::spawn(|| {
            process_plan(process_receiver, update_sender);
        }));
        Engine::singleton()
            .register_singleton(Self::singleton_name(), ai_manager.clone());
        // ai_manager.call_deferred("post_ready".into(), &[]);
        ai_manager
    }

    fn exit(&mut self) {
        Engine::singleton().unregister_singleton(Self::singleton_name());
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(ThinkerPlanEvent::Terminate);
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let mut memories: Vec<_> = self.thinkers.values().map(|t| t.shared.clone()).collect();
        memories.par_iter_mut().rev().for_each(|shared| {
            let Ok(mut shared) = shared.lock() else {
                panic!("Couldn't read thinker working memory")
            };
            shared.working_memory.validate();
        });
        drop(memories);

        for thinker in self.thinkers.values_mut() {
            if !thinker.is_active {
                continue;
            }
            process_thinker(thinker, delta, &self.ai_nodes);
            if let Some(sender) = self.sender.as_mut() {
                let _result = sender.send(ThinkerPlanEvent::Process(
                    ThinkerProcess::from(&*thinker).with_ainodes(self.ai_nodes.clone()),
                ));
            }
        }
    }
}