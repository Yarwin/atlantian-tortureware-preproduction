use crate::ai::world_state::WorldState;
use pathfinding::prelude::astar;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// a generic trait that must be implemented by any GoapAction
pub trait PlanAction<U>: PartialEq<Self> + Hash + Debug {
    fn get_action_preconditions(&self) -> &WorldState;
    fn check_action_procedural_preconditions(&self, action_arguments: &U) -> bool;
    fn get_action_effects<'a, 'b: 'a>(&'a self, action_arguments: &'b U) -> &'a WorldState;
    fn get_action_cost(&self, action_arguments: &U) -> u32;
}

#[derive(Debug)]
pub struct PlanNode<'a, T: PlanAction<U>, U> {
    pub current_state: WorldState,
    pub action: Option<&'a T>,
    extra_action_arguments: PhantomData<U>,
}

impl<'a, T: PlanAction<U>, U> Clone for PlanNode<'a, T, U> {
    fn clone(&self) -> Self {
        PlanNode {
            current_state: self.current_state.clone(),
            action: self.action,
            extra_action_arguments: PhantomData,
        }
    }
}

impl<T: PlanAction<U>, U> Eq for PlanNode<'_, T, U> {}

impl<'a, T: PlanAction<U>, U> PartialEq<Self> for PlanNode<'a, T, U> {
    fn eq(&self, other: &Self) -> bool {
        if self.action.is_some() && other.action.is_some() {
            return self.action.as_ref().unwrap() == other.action.as_ref().unwrap();
        }
        false
    }
}

impl<T: PlanAction<U>, U> Hash for PlanNode<'_, T, U> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(action) = self.action.as_ref() {
            action.hash(state);
        }
        self.current_state.hash(state);
    }
}

impl<'a, T: PlanAction<U>, U> PlanNode<'a, T, U> {
    pub fn initial(initial_state: &WorldState) -> PlanNode<T, U> {
        PlanNode {
            current_state: initial_state.clone(),
            action: None,
            extra_action_arguments: PhantomData,
        }
    }

    pub fn child(
        parent_state: WorldState,
        action_type: &'a T,
        action_arguments: &'a U,
    ) -> PlanNode<'a, T, U> {
        let mut child = PlanNode {
            current_state: parent_state.clone(),
            action: Some(action_type),
            extra_action_arguments: PhantomData,
        };
        child
            .current_state
            .apply_world_state(action_type.get_action_effects(action_arguments));
        child
    }

    pub fn neighbours(
        &self,
        actions: &'a [T],
        action_arguments: &'a U,
    ) -> Vec<(PlanNode<'a, T, U>, u32)> {
        let actions = actions
            .iter()
            .filter_map(|action| {
                // effects are already satisfied in current state
                if action
                    .get_action_effects(action_arguments)
                    .count_unsatisfied_world_state_props(&self.current_state)
                    == 0
                {
                    return None;
                }

                // preconditions are not met
                if action
                    .get_action_preconditions()
                    .count_unsatisfied_world_state_props(&self.current_state)
                    != 0
                {
                    return None;
                }

                if !action.check_action_procedural_preconditions(action_arguments) {
                    return None;
                }

                Some((
                    PlanNode::child(self.current_state.clone(), action, action_arguments),
                    action.get_action_cost(action_arguments),
                ))
            })
            .collect();
        actions
    }
}

pub fn plan<'a, T: PlanAction<U>, U>(
    initial_state: &'a WorldState,
    goal_state: &'a WorldState,
    actions: &'a [T],
    action_arguments: &'a U,
) -> Option<Vec<&'a T>> {
    let start = PlanNode::initial(initial_state);
    let plan = astar(
        &start,
        |node| node.neighbours(actions, action_arguments),
        |node| node.current_state.count_state_differences(goal_state),
        |node| goal_state.count_unsatisfied_world_state_props(&node.current_state) == 0,
    );
    if let Some((plan, _cost)) = plan {
        // skip a root node (deferred from a goal)
        return Some(
            plan.into_iter()
                .skip(1)
                .map(|node| node.action.unwrap())
                .collect(),
        );
    }
    None
}

#[cfg(test)]
#[cfg(feature = "use_serde")]
mod test {
    use super::*;
    use serde_derive::Deserialize;
    use std::fs;
    use std::path::Path;
    extern crate serde_json;
    #[derive(Deserialize, Hash, Clone, Debug)]
    struct TestAction {
        name: String,
        preconditions: WorldState,
        effects: WorldState,
        cost: u32,
    }

    impl PartialEq<Self> for TestAction {
        fn eq(&self, other: &Self) -> bool {
            if self.name == other.name {
                return true;
            }
            return false;
        }
    }

    impl PlanAction<bool> for TestAction {
        fn get_action_preconditions(&self) -> &WorldState {
            &self.preconditions
        }

        fn check_action_procedural_preconditions(&self, action_arguments: &bool) -> bool {
            *action_arguments
        }

        fn get_action_effects(&self) -> &WorldState {
            &self.effects
        }

        fn get_action_cost(&self) -> u32 {
            self.cost
        }
    }

    #[derive(Deserialize)]
    struct TestCase {
        #[serde(skip_deserializing)]
        case_name: String,
        actions: Vec<TestAction>,
        initial_state: WorldState,
        goal_state: WorldState,
        expected_actions: Vec<String>,
    }
    impl TestCase {
        fn from_test_file(path: &Path) -> TestCase {
            let file = fs::File::open(path).unwrap();
            let mut case: TestCase = serde_json::from_reader(file).unwrap();
            case.case_name = String::from(path.file_name().unwrap().to_str().unwrap());
            case
        }
        fn assert_plan(&self) {
            let plan = plan(&self.initial_state, &self.goal_state, &self.actions, &true);
            if let Some(action_list) = plan {
                let action_names: Vec<String> = action_list
                    .iter()
                    .map(|&action| action.name.clone())
                    .collect();
                if self.expected_actions != action_names {
                    panic!(
                        "{} failed: expected {:?}, got {:?}",
                        self.case_name, self.expected_actions, action_names
                    );
                }
            } else {
                if self.expected_actions.len() > 0 {
                    panic!(
                        "{} failed: expected {:?}, got no plan",
                        self.case_name, self.expected_actions
                    );
                }
            }
        }
    }

    #[test]
    fn run_test_files() {
        let paths = fs::read_dir("./test_data/plan_test").unwrap();
        for path in paths {
            let case = TestCase::from_test_file(path.unwrap().path().as_path());
            case.assert_plan();
        }
    }
}
