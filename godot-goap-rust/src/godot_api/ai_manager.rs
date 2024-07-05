use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use godot::prelude::*;
use godot::classes::{FileAccess, Engine};
use godot::engine::file_access::ModeFlags;
use serde::Deserialize;
use crate::ai::thinker::Thinker;
use crate::ai::process_plan::{process_plan, ThinkerPlanEvent, ThinkerProcess};
use crate::goals::goal_component::GoalComponent;
use crate::godot_api::godot_thinker::GodotThinker;
use std::sync::mpsc;
use rayon::prelude::*;
use crate::actions::action_types::Action;
use crate::ai_nodes::ai_node::{AINode};
use crate::ai_nodes::godot_ai_node::GodotAINode;
use crate::animations::animation_data::AnimationsData;
use crate::godot_api::CONNECT_ONE_SHOT;
use crate::sensors::sensor_types::PollingSensor;
use crate::thinker_states::process_thinker::process_thinker;

#[derive(GodotClass)]
#[class(init, base=Node, rename=AIManager)]
pub struct GodotAIManager {
    pub actions: HashMap<GString, Arc<Vec<Action>>>,
    pub goals: HashMap<GString, Arc<Vec<GoalComponent>>>,
    pub animations: HashMap<GString, Arc<AnimationsData>>,
    sensors_blueprint: HashMap<GString, Vec<PollingSensor>>,
    pub ai_nodes: Arc<RwLock<HashMap<u32, Arc<Mutex<AINode>>>>>,
    ainode_id_with_dependencies: VecDeque<(u32, Gd<GodotAINode>)>,

    pub thinkers: HashMap<u32, Thinker>,
    pub current_thinker_id: u32,
    pub current_node_id: u32,
    pub sender: Option<Sender<ThinkerPlanEvent>>,
    pub receiver: Option<Receiver<()>>,
    pub thread: Option<thread::JoinHandle<()>>,
    base: Base<Node>,
}


#[godot_api]
impl INode for GodotAIManager {

    fn physics_process(&mut self, delta: f64) {
        let mut memories: Vec<_> = self.thinkers.values().map(|t| t.shared.clone()).collect();
        memories.par_iter_mut().for_each(|shared|{
            let Ok(mut shared) = shared.lock() else {panic!("mutex failed")};
            shared.working_memory.increase_time(delta);
            shared.working_memory.validate();
        });
        drop(memories);

        for thinker in self.thinkers.values_mut() {
            if !thinker.is_active {
                continue
            }
            process_thinker(thinker, delta, &self.ai_nodes);
            if let Some(sender) = self.sender.as_mut() {
                let _result = sender.send(ThinkerPlanEvent::Process(ThinkerProcess::from(&*thinker).with_ainodes(self.ai_nodes.clone())));
            }
        }
    }

    fn enter_tree(&mut self) {
        Engine::singleton().register_singleton(
            "AIManager".into(),
            self.base().clone().upcast::<Object>(),
        );
    }

    fn exit_tree(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(ThinkerPlanEvent::Terminate);
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }

    /// starts a Thinker/planner thread and registers all the API consumers
    fn ready(&mut self) {
        let (process_sender, process_receiver) = mpsc::channel();
        let (update_sender, update_receiver) = mpsc::channel();
        self.sender = Some(process_sender);
        self.receiver = Some(update_receiver);
        // spawns a new thread ready to accept events
        self.thread = Some(thread::spawn(|| {
            process_plan(process_receiver, update_sender);
        }));
        self.base_mut().call_deferred("post_ready".into(), &[]);
    }
}


#[godot_api]
impl GodotAIManager {
    #[func]
    fn post_ready(&mut self) {
        // update all dependencies
        for (node_id, dependency) in self.ainode_id_with_dependencies.drain(..) {
            let dep_node_id = dependency.bind().ainode_id;
            let Ok(mut ai_nodes_writer) = self.ai_nodes.write() else {godot_print!("rw lock failed!"); panic!("RwLock Writer failed!");};
            let Some(ainode_guard) = ai_nodes_writer.get_mut(&node_id) else {panic!("no such ainode!")};
            let Ok(mut ainode) = ainode_guard.lock() else {panic!("Mutex failed!")};
            let new_ainode = std::mem::take(&mut *ainode);
            *ainode = new_ainode.with_dependency(dep_node_id);
        }
    }

    #[func]
    fn unregister_thinker(&mut self, id: u32) {
        self.thinkers.remove(&id);
    }

    #[func]
    fn unregister_ainode(&mut self, id: u32) {
        let Ok(mut ai_nodes) = self.ai_nodes.write() else {godot_print!("rw lock failed!"); panic!("RwLock Writer failed!");};
        ai_nodes.remove(&id);
    }
}


impl GodotAIManager {
    fn get_ainode_id(&mut self) -> u32 {
        self.current_node_id += 1;
        self.current_node_id
    }

    /// increases current_node_id to prevent overwriting already set ainodes
    fn ainode_id_to_max(&mut self, other: u32) {
        if self.current_node_id > other {return;}
        self.current_node_id = other;
    }

    fn get_thinker_id(&mut self) -> u32 {
        self.current_thinker_id += 1;
        self.current_thinker_id
    }

    /// increases current_thinker_id to prevent overwriting already set thinkers
    fn thinker_id_to_max(&mut self, other: u32) {
        if self.current_thinker_id > other {return;}
        self.current_thinker_id = other;
    }

    pub fn register_ainode(&mut self, ai_node: &mut GodotAINode) -> u32 {
        let id: u32;
        if ai_node.ainode_id == 0 {
            id = self.get_ainode_id();
            ai_node.ainode_id = id;
        } else {
            id = ai_node.ainode_id;
            self.ainode_id_to_max(ai_node.ainode_id);
        }
        // unregister on exit
        let callable = Callable::from_object_method(&self.base().clone(), "unregister_ainode").bindv(array![id.to_variant()]);
        let _ = ai_node.base_mut().connect_ex("tree_exiting".into(), callable).flags(CONNECT_ONE_SHOT).done();
        // update dependencies in next cycle
        if ai_node.dependency.is_some() {
            self.ainode_id_with_dependencies.push_front((id, ai_node.dependency.as_ref().unwrap().clone()));
        }
        let node = AINode::from(&*ai_node);
        let Ok(mut ai_nodes) = self.ai_nodes.write() else {panic!("RwLock Writer failed!");};
        ai_nodes.insert(id, Arc::new(Mutex::new(node)));
        id
    }

    pub fn register_thinker(&mut self, godot_thinker: &mut GodotThinker) -> u32 {
        let id: u32;
        if godot_thinker.thinker_id == 0 {
            id = self.get_thinker_id();
        } else {
            id = godot_thinker.thinker_id;
            self.thinker_id_to_max(godot_thinker.thinker_id)
        }

        // unregister on exit
        let callable = Callable::from_object_method(&self.base().clone(), "unregister_thinker").bindv(array![id.to_variant()]);
        let _ = godot_thinker.base_mut().connect_ex("tree_exiting".into(), callable).flags(CONNECT_ONE_SHOT).done();
        let navigation_map_rid = godot_thinker.navigation_agent.as_ref().map(|agent| agent.get_navigation_map());

        let thinker = Thinker {
            id,
            base: Some(godot_thinker.base().clone().cast::<GodotThinker>()),
            is_active: godot_thinker.is_active,
            actions: self.get_actions(&godot_thinker.actions_file).unwrap(),
            goals: self.get_goals(&godot_thinker.goals_file).unwrap(),
            polling_sensors: self.get_sensors(&godot_thinker.sensors_file).unwrap(),
            animations: self.get_animations_data(&godot_thinker.animation_data).unwrap(),
            navigation_map_rid,
            ..Default::default()
        };
        thinker.shared.lock().unwrap().blackboard.thinker_position = godot_thinker.base().get_global_position();
        self.thinkers.insert(id, thinker);
        id
    }

    fn load<T: for<'a> Deserialize<'a>>(path: &GString) -> T {
        let file = FileAccess::open(path.clone(), ModeFlags::READ);
        file.as_ref().expect("Couldn't read given component config!");
        ron::from_str::<T>(&String::from(file.unwrap().get_as_text()))
            .expect("Couldn't read given component config!")
    }

    /// loads & caches given set of components
    fn load_components<T, U: for<'a> Deserialize<'a> + Into<T>>(collection: &mut HashMap<GString, Arc<Vec<T>>>, path: &GString) -> Option<Arc<Vec<T>>> {
        if let Some(collection) = collection.get(path) {
            return Some(collection.clone())
        }
        let components: Arc<Vec<T>> = Arc::new(Self::load::<Vec<U>>(path).into_iter().map(Into::into).collect());
        collection.insert(path.clone(), components.clone());
        Some(components)
    }

    pub fn get_actions(&mut self, path: &GString) -> Option<Arc<Vec<Action>>> {
        Self::load_components::<Action, Action>(&mut self.actions, path)
    }

    pub fn get_goals(&mut self, path: &GString) -> Option<Arc<Vec<GoalComponent>>> {
        Self::load_components::<GoalComponent, GoalComponent>(&mut self.goals, path)
    }

    pub fn get_animations_data(&mut self, path: &GString) -> Option<Arc<AnimationsData>> {
        if let Some(collection) = self.animations.get(path) {
            return Some(collection.clone())
        }
        let components: Arc<AnimationsData> = Arc::new(Self::load::<AnimationsData>(path));
        self.animations.insert(path.clone(), components.clone());
        Some(components)
    }

    fn get_sensors(&mut self, path: &GString) -> Option<Vec<PollingSensor>> {
        if let Some(collection) = self.sensors_blueprint.get(path) {
            return Some(collection.clone())
        }
        let components: Vec<PollingSensor> = Self::load(path);
        self.sensors_blueprint.insert(path.clone(), components.clone());
        Some(components)
    }
}
