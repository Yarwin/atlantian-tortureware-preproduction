use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use godot::builtin::Rid;
use godot::global::godot_print;
use godot::obj::InstanceId;
use crate::actions::action_types::{Action};
use crate::ai::blackboard::Blackboard;
use crate::ai::planner::plan;
use crate::ai::thinker::{Thinker, ThinkerShared};
use crate::ai::working_memory::WorkingMemory;
use crate::ai::world_state::WorldState;
use crate::goals::goal_component::GoalComponent;
use crate::goals::goal_types::GoalBehaviour;
use crate::{action_arguments, action_plan_context, thinker_process_to_goal_view};
use crate::actions::action_types::ActionBehavior;
use crate::ai_nodes::ai_node::AINode;
use crate::animations::animation_data::AnimationsData;


#[derive(Debug)]
pub enum ThinkerPlanEvent {
    Process(ThinkerProcess),
    Terminate,
}


#[derive(Debug)]
pub enum PlanMessage {

}


impl ThinkerPlanEvent {
    fn terminate(&self) -> bool {
        matches!(self, ThinkerPlanEvent::Terminate)
    }
    fn process_view(self) -> ThinkerProcess {
        match self {
            ThinkerPlanEvent::Process(v) => {v}
            ThinkerPlanEvent::Terminate => {panic!("aaa")}
        }
    }
}

/// all the information about our Thinker sent via channel
#[derive(Default, Debug)]
pub struct ThinkerProcess {
    pub id: u32,
    pub base_id: Option<InstanceId>,
    pub shared: Arc<Mutex<ThinkerShared>>,
    // pub blackboard: Arc<Mutex<Blackboard>>,
    pub actions: Arc<Vec<Action>>,
    pub goals: Arc<Vec<GoalComponent>>,
    pub animations: Arc<AnimationsData>,
    // pub working_memory: Arc<Mutex<WorkingMemory>>,
    // pub world_state: Arc<Mutex<WorldState>>,
    pub navigation_map_rid: Option<Rid>,
    pub ai_nodes: Option<Arc<RwLock<HashMap<u32, Arc<Mutex<AINode>>>>>>,
}

pub struct ThinkerPlanView<'a> {
    pub id: &'a u32,
    // pub shared: &'a mut ThinkerShared,
    pub goals: &'a Arc<Vec<GoalComponent>>,
    pub actions: &'a Arc<Vec<Action>>,
    pub animations: &'a Arc<AnimationsData>,
    pub navigation_map_rid: &'a Option<Rid>,
    pub ai_nodes: &'a Option<Arc<RwLock<HashMap<u32, Arc<Mutex<AINode>>>>>>,
    pub blackboard: &'a mut Blackboard,
    pub working_memory: &'a mut WorkingMemory,
    pub world_state: &'a mut WorldState
}


impl From<&Thinker> for ThinkerProcess {
    fn from(value: &Thinker) -> Self {
        ThinkerProcess {
            id: value.id,
            base_id: value.base_id,
            shared: value.shared.clone(),
            goals: value.goals.clone(),
            actions: value.actions.clone(),
            navigation_map_rid: value.navigation_map_rid,
            ai_nodes: None,
            animations: value.animations.clone()
        }
    }
}

impl ThinkerProcess {
    pub fn with_ainodes(mut self, ainodes: Arc<RwLock<HashMap<u32, Arc<Mutex<AINode>>>>>) -> Self {
        self.ai_nodes = Some(ainodes);
        self
    }
}


fn get_relevant_goal(thinker: &mut ThinkerPlanView) -> Option<usize> {
    let current_goal: Option<usize> = thinker.blackboard.current_goal;
    let mut best_priority: u32 = 0;
    let context = thinker_process_to_goal_view!(thinker);
    if let Some(current) = current_goal {
        best_priority = thinker.goals[current].goal_type.calculate_goal_relevance(&thinker.goals[current], &context);
    }

    let mut best_goal: Option<usize> = None;

    for (id, goal) in thinker.goals.iter().enumerate() {
        // bail if world state doesn't match
        if !goal.goal_type.validate_context(goal, &context) {
            continue
        }
        // bail if goal is not valid
        if !goal.goal_type.is_valid(goal, &context) {
            continue
        }

        let priority: u32 = goal.goal_type.calculate_goal_relevance(goal, &context);
        if priority > best_priority {
            best_priority = priority;
            best_goal = Some(id);
        }
    }
    best_goal
}


/// recalculate the best goal and plan
fn update_plan(thinker_view: &mut ThinkerPlanView) -> Option<(VecDeque<usize>, usize)> {
    let current_goal: Option<usize> = None;

    let new_goal = get_relevant_goal(thinker_view)?;

    // bail if no goal change. Replan if plan is empty.
    if let Some(equality) = current_goal.map(|g| new_goal == g) {
        if equality {
            return None;
        }
    }

    // bail if goal can't be activated
    if !activate_new_goal(thinker_view, new_goal) {
        return None
    }

    let initial_state = thinker_view.world_state.clone();
    let action_arguments = action_plan_context!(thinker_view);
    // get a plan
    let some_plan = plan(&initial_state, &thinker_view.goals[new_goal].desired_state, thinker_view.actions, &action_arguments);
    if let Some(plan) = some_plan {
        let indexes: VecDeque<usize> = plan.iter().filter_map(
            |step| thinker_view.actions.iter().position(|a| a == *step)
        ).collect();
        return Some((indexes, new_goal))
    }
    None
}


/// called when one of the action has been completed.
/// Advances the plan.
fn advance_plan(thinker_view: &mut ThinkerPlanView) {
    loop {
        // remove action from the blackboard
        let current_action: Option<usize> = thinker_view.blackboard.next_action();

        // finalize action
        if let Some(index) = current_action {
            thinker_view.actions[index].finish(action_arguments!(thinker_view));
        } else {
            return;
        }

        let current_action: Option<usize> = thinker_view.blackboard.current_action();
        // execute the current action
        if let Some(index) = current_action {
            thinker_view.actions[index].execute_action(action_arguments!(thinker_view));
            // advance the plan if action has been completed instantly
            let action_arguments = action_arguments!(thinker_view);
            if !thinker_view.actions[index].is_action_complete(&action_arguments) {
                return;
            }
        } else {
            // No more actions. Deactivate the goal.
            let goal_id = thinker_view.blackboard.current_goal.take().unwrap();
            let mut goal_args = thinker_process_to_goal_view!(thinker_view);
            thinker_view.goals[goal_id].goal_type.deactivate(&thinker_view.goals[goal_id], &mut goal_args);
            return;
        }
    }
}


/// activates new goal
/// returns false if goal can not be activated
fn activate_new_goal(thinker_view: &mut ThinkerPlanView, new_goal: usize) -> bool {
    if let Some(action) = thinker_view.blackboard.current_action() {
        let action_arguments = action_arguments!(thinker_view);
        thinker_view.actions[action].finish(action_arguments);
    }

    // deactivate previous goal
    let mut previous_goal: Option<&GoalComponent> = None;
    if let Some(old_goal) = thinker_view.blackboard.current_goal {
        previous_goal = Some(&thinker_view.goals[old_goal]);
    }
    let mut context = thinker_process_to_goal_view!(thinker_view);
    if let Some(previous) = previous_goal {
        previous.goal_type.deactivate(previous, &mut context);
    }

    // activate goal
    thinker_view.goals[new_goal].goal_type.activate(&thinker_view.goals[new_goal], &mut context)
}


/// finalizes previous plan and activates the new one
fn activate_plan(thinker_view: &mut ThinkerPlanView, new_plan: VecDeque<usize>, new_goal: usize) {
    let first_action: usize = new_plan[0];

    // update blackboard
    {
        thinker_view.blackboard.current_goal = Some(new_goal);
        thinker_view.blackboard.current_plan_ids = new_plan;
    }

    let action_arguments = action_arguments!(thinker_view);
    thinker_view.actions[first_action].execute_action(action_arguments);
    let action_arguments = action_arguments!(thinker_view);
    // advance the plan if it was finished imminently
    if thinker_view.actions[first_action].is_action_complete(&action_arguments) {
        advance_plan(thinker_view);
    }
}


fn process_goal_and_plan(event: ThinkerPlanEvent) {
    let thinker_process = event.process_view();
    let Ok(mut shared_lock) = thinker_process.shared.lock() else { panic!("mutex failed!") };
    // deref guard to inner to make mut references to said struct
    let shared = &mut *shared_lock;
    let (blackboard, world_state, working_memory) = (&mut shared.blackboard, &mut shared.world_state, &mut shared.working_memory);
    let mut thinker_process_view = ThinkerPlanView {
        id: &thinker_process.id,
        goals: &thinker_process.goals,
        actions: &thinker_process.actions,
        animations: &thinker_process.animations,
        navigation_map_rid: &thinker_process.navigation_map_rid,
        ai_nodes: &thinker_process.ai_nodes,
        blackboard,
        world_state,
        working_memory
    };

    let new_plan = update_plan(&mut thinker_process_view);

    if let Some((new_p, new_g)) = new_plan {
        activate_plan(&mut thinker_process_view, new_p, new_g);
    }
    let current_action: Option<usize> = thinker_process_view.blackboard.current_action();

    if let Some(action) = current_action {
        let action_arguments = action_arguments!(thinker_process_view);
        if thinker_process.actions[action].is_action_complete(&action_arguments) {
            advance_plan(&mut thinker_process_view);
        }
    }
}


pub fn process_plan(receiver: Receiver<ThinkerPlanEvent>, _sender: Sender<()>) {
    loop {
        if let Ok(message) = receiver.recv() {
            if message.terminate() {
                godot_print!("terminating…");
                break;
            }
            process_goal_and_plan(message)
        }
    }
}
