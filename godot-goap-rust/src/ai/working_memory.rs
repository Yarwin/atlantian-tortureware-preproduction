use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use godot::prelude::*;
use strum_macros::EnumDiscriminants;
use crate::ai_nodes::ai_node::AINode;

/// AIWorking memory is a central place to store the AI's observations about the world.
/// AISensors and AIGoals publish and retrieve data to/from AIWorkingMemory to make decisions.

#[derive(Debug, PartialEq, Eq, EnumDiscriminants)]
#[strum_discriminants(name(WorkingMemoryFactKnowledgeTypeKey))]
pub enum KnowledgeType {
    Invalid
}

#[derive(Debug, PartialEq, Eq, EnumDiscriminants)]
#[strum_discriminants(name(WorkingMemoryFactValueDesireTypeKey))]
pub enum DesireType {
    Invalid,
    Stun,
    Stagger,
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(name(WorkingMemoryFactValueTaskTypeKey))]
pub enum TaskType {
    Cover,
    Advance,
}


#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(name(WorkingMemoryFactValueNodeTypeKey))]
pub enum NodeType {
    Patrol{ainode: Arc<Mutex<AINode>>, position: Vector3},
}


#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(name(WorkingMemoryFactTypeKey))]
pub enum WorkingMemoryFactType {
    Invalid,
    Character(InstanceId),
    Damage{amount: f64, direction: Vector3},
    Desire(DesireType),
    Disturbance,
    Node(NodeType),
    Task(TaskType),
    Knowledge
}


#[derive(Debug)]
pub struct WorkingMemoryFact {
    /// a value in range of 0-100 telling about importance/confidence of a given fact
    pub confidence: f32,
    pub id: u32,
    pub f_type: WorkingMemoryFactType,
    /// time since initialization at the time of adding this fact
    pub update_time: f64,
    expiration: f64,
    pub is_valid: bool,
}

impl WorkingMemoryFact {
    pub fn matches_query(&self, other: &FactQuery) -> bool {
        for check in other.checks.iter() {
            match check {
                FactQueryCheck::UpdateTime(elapsed) => {
                    if self.update_time > *elapsed {
                        return false;
                    }
                }
                FactQueryCheck::FactId(id) => {
                    if self.id != *id {
                        return false;
                    }
                }
                FactQueryCheck::FactType(fact_type) => {
                    if WorkingMemoryFactTypeKey::from(&self.f_type) != *fact_type {
                        return false;
                    }
                }
                FactQueryCheck::NodeValue(node_type) => {
                    if let WorkingMemoryFactType::Node(v) = &self.f_type {
                        if WorkingMemoryFactValueNodeTypeKey::from(v) != *node_type {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                FactQueryCheck::TaskType(task_type) => {
                    if let WorkingMemoryFactType::Task(t) = &self.f_type {
                        if WorkingMemoryFactValueTaskTypeKey::from(t) != *task_type {
                            return false;
                        }
                    }  else {
                        return false
                    }
                }
                FactQueryCheck::Character(character_id) => {
                    if let WorkingMemoryFactType::Character(other_id) = &self.f_type {
                        if other_id == character_id {
                            return true
                        }
                    }
                    return false
                }
            }
        }
        true
    }
}


#[derive(Debug)]
pub struct WorkingMemory {
    /// time elapsed since initialization
    pub elapsed_time: f64,
    current_id: u32,
    capacity: usize,
    facts_list: VecDeque<WorkingMemoryFact>,
    /// a queue that holds a list of facts to remove/replace
    to_remove: VecDeque<usize>,
}


impl Default for WorkingMemory {
    fn default() -> Self {
        let capacity = 32;
        WorkingMemory {
            elapsed_time: 0.0,
            current_id: 0,
            capacity,
            facts_list: VecDeque::with_capacity(capacity),
            to_remove: Default::default(),
        }
    }
}

impl WorkingMemory {
    fn next_id(&mut self) -> u32 {
        self.current_id += 1;
        self.current_id
    }

    pub fn with_capacity(capacity: usize) -> Self {
        WorkingMemory {
            elapsed_time: 0.0,
            current_id: 0,
            capacity,
            facts_list: VecDeque::with_capacity(capacity),
            to_remove: Default::default(),
        }
    }

    pub fn increase_time(&mut self, delta: f64) {
        self.elapsed_time += delta;
    }

    pub fn add_working_memory_fact(&mut self, f_type: WorkingMemoryFactType, confidence: f32, expiration: f64) {
        let id = self.next_id();
        let fact = WorkingMemoryFact {
            confidence,
            id,
            f_type,
            update_time: self.elapsed_time,
            expiration,
            is_valid: true,
        };
        if let Some(index) = self.to_remove.pop_back() {
            self.facts_list[index] = fact;
        } else {
            self.facts_list.push_front(fact);
        }
    }


    pub fn count_facts(&self, query: FactQuery) -> u32 {
        let mut count: u32 = 0;
        for fact in self.facts_list.iter() {
            if fact.matches_query(&query) {
                count += 1;
            }
        }

        count
    }

    // marks facts as invalid
    pub fn validate(&mut self) {
        let elapsed_time = self.elapsed_time;
        self.to_remove.extend(
            self.facts_list.iter_mut().enumerate().filter_map(|(i, f)| {
                if ((f.update_time + f.expiration) < elapsed_time) && f.is_valid {
                    f.is_valid = false;
                    return Some(i)
                }
                None
            })
        );

    }

    fn facts(&self) -> impl Iterator<Item=&WorkingMemoryFact> {
        self.facts_list.iter().filter(|f| f.is_valid)
    }

    fn facts_mut(&mut self) -> impl Iterator<Item=&mut WorkingMemoryFact> {
        self.facts_list.iter_mut().filter_map(|f| {
            if !f.is_valid {
                return None
            }
            Some(f)
        })
    }

    pub fn find_fact(&self, query: FactQuery) -> Option<&WorkingMemoryFact> {
        self.facts().find(|&fact| fact.matches_query(&query))
    }

    pub fn find_fact_mut(&mut self, query: FactQuery) -> Option<&mut WorkingMemoryFact> {
        self.facts_mut().find(|fact| fact.matches_query(&query))
    }

    /// marks given fact as invalid and returns mutable reference
    pub fn find_and_mark_as_invalid(&mut self, query: FactQuery) -> Option<&mut WorkingMemoryFact> {
        let fact_index = self.facts_list.iter().position(|fact| fact.matches_query(&query))?;
        self.to_remove.push_back(fact_index);
        let fact = &mut self.facts_list[fact_index];
        fact.is_valid = false;
        Some(fact)
    }
}


pub enum FactQueryCheck {
    FactId(u32),
    UpdateTime(f64),
    FactType(WorkingMemoryFactTypeKey),
    NodeValue(WorkingMemoryFactValueNodeTypeKey),
    TaskType(WorkingMemoryFactValueTaskTypeKey),
    Character(InstanceId)
}

#[derive(Default)]
pub struct FactQuery {
    pub checks: Vec<FactQueryCheck>
}

impl FactQuery {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_check(check: FactQueryCheck) -> Self {
        let mut query = Self::new();
        query.checks.push(check);
        query
    }

    pub fn check(mut self, check: FactQueryCheck) -> Self {
        self.checks.push(check);
        self
    }
}