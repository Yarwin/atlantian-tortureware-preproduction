use crate::receiver::damage_receptor_component::ReceivedDamage;
use godot::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::SystemTime;
use strum_macros::EnumDiscriminants;

/// AIWorking memory is a central place to store the AI's observations about the world.
/// AISensors and AIGoals publish and retrieve data to/from AIWorkingMemory to make decisions.

#[derive(Debug, EnumDiscriminants, Clone)]
#[strum_discriminants(name(WMKnowledgeType))]
pub enum Knowledge {
    Invalid,
    Character(InstanceId, Option<Vector3>),
    LastTargetPosition(Vector3),
}

impl Eq for Knowledge {}

impl PartialEq for Knowledge {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Knowledge::Character(i, ..), Knowledge::Character(other_i, ..)) => i == other_i,
            (Knowledge::LastTargetPosition(pos), Knowledge::LastTargetPosition(other_pos)) => {
                (*pos - *other_pos).is_zero_approx()
            }
            (_, _) => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, EnumDiscriminants, Clone)]
#[strum_discriminants(name(WMDesireType))]
#[strum_discriminants(derive(Serialize, Deserialize))]
pub enum Desire {
    Invalid,
    Stun,
    Stagger,
    Surprise,
    Death,
}

#[derive(Debug, PartialEq, Eq, EnumDiscriminants, Clone)]
#[strum_discriminants(name(WMTaskType))]
pub enum Task {
    Cover,
    Advance,
}

#[derive(Debug, EnumDiscriminants, Clone)]
#[strum_discriminants(name(WMNodeType))]
pub enum Node {
    Patrol { ainode_id: u32, position: Vector3 },
}

impl Eq for Node {}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (&self, other) {
            (
                Node::Patrol { ainode_id, .. },
                Node::Patrol {
                    ainode_id: other_ainode_id,
                    ..
                },
            ) => ainode_id == other_ainode_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, EnumDiscriminants)]
#[strum_discriminants(name(WMEventType))]
pub enum Event {
    AnimationCompleted(String),
    AttackPerformed { id: usize },
    AttackFailed { id: usize },
    GoalFailed { id: usize },
}

#[derive(Clone, Debug, EnumDiscriminants)]
#[strum_discriminants(name(WMAIStimuliType))]
pub enum AIStimuli {
    /// visible character stimuli
    Character(InstanceId, Option<Vector3>),
    Damage(ReceivedDamage),
}

impl Eq for AIStimuli {}

impl PartialEq for AIStimuli {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (AIStimuli::Character(i, ..), AIStimuli::Character(other_i, ..)) => {
                if i == other_i {
                    return true;
                }
                false
            }
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, EnumDiscriminants)]
#[strum_discriminants(name(WorkingMemoryFactTypeKey))]
pub enum WMProperty {
    AIStimuli(AIStimuli),
    Desire(Desire),
    Disturbance,
    Node(Node),
    Task(Task),
    Knowledge(Knowledge),
    Event(Event),
}

#[derive(Debug)]
pub struct WorkingMemoryFact {
    /// a value in range of 0-100 telling about importance/confidence of a given fact
    pub confidence: f32,
    pub f_type: WMProperty,
    /// time since initialization at the time of adding this fact
    pub update_time: SystemTime,
    expiration: f64,
    is_valid: bool,
}

impl WorkingMemoryFact {
    pub fn matches_query(&self, other: &FactQuery) -> bool {
        for check in other.checks.iter() {
            match check {
                FactQueryCheck::Node(node_type) => {
                    let WMProperty::Node(v) = &self.f_type else {
                        return false;
                    };
                    if WMNodeType::from(v) != *node_type {
                        return false;
                    }
                }
                FactQueryCheck::TaskType(task_type) => {
                    let WMProperty::Task(t) = &self.f_type else {
                        return false;
                    };
                    if WMTaskType::from(t) != *task_type {
                        return false;
                    }
                }
                FactQueryCheck::Knowledge(knowledge_type) => {
                    let WMProperty::Knowledge(k) = &self.f_type else {
                        return false;
                    };
                    if WMKnowledgeType::from(k) != *knowledge_type {
                        return false;
                    }
                }
                FactQueryCheck::Desire(desire_type) => {
                    let WMProperty::Desire(d) = &self.f_type else {
                        return false;
                    };
                    if WMDesireType::from(d) != *desire_type {
                        return false;
                    }
                }
                FactQueryCheck::Event(e_type) => {
                    let WMProperty::Event(e) = &self.f_type else {
                        return false;
                    };
                    if WMEventType::from(e) != *e_type {
                        return false;
                    }
                }
                FactQueryCheck::AIStimuli(s_type) => {
                    let WMProperty::AIStimuli(s) = &self.f_type else {
                        return false;
                    };
                    if WMAIStimuliType::from(s) != *s_type {
                        return false;
                    }
                }
                FactQueryCheck::Match(wmfact_type) => {
                    if wmfact_type != &self.f_type {
                        return false;
                    }
                }
            }
        }
        true
    }
}

#[derive(Debug)]
pub struct WorkingMemory {
    clean_threshold: usize,
    facts_list: VecDeque<WorkingMemoryFact>,
    /// a queue that holds a list of facts to remove/replace
    /// todo - use binary heap instead?
    to_remove: VecDeque<usize>,
}

impl Default for WorkingMemory {
    fn default() -> Self {
        WorkingMemory {
            clean_threshold: 26,
            facts_list: VecDeque::with_capacity(32),
            to_remove: Default::default(),
        }
    }
}

impl WorkingMemory {
    pub fn with_capacity(capacity: usize) -> Self {
        WorkingMemory {
            clean_threshold: (capacity as f32 * 0.8) as usize,
            facts_list: VecDeque::with_capacity(capacity),
            to_remove: Default::default(),
        }
    }

    pub fn add_working_memory_fact(
        &mut self,
        f_type: WMProperty,
        confidence: f32,
        expiration: f64,
    ) {
        let fact = WorkingMemoryFact {
            confidence,
            f_type,
            update_time: SystemTime::now(),
            expiration,
            is_valid: true,
        };
        if let Some(index) = self.to_remove.pop_back() {
            self.facts_list[index] = fact;
        } else {
            self.facts_list.push_front(fact);
        }
    }

    pub fn add_or_update(&mut self, f_type: WMProperty, confidence: f32, expiration: f64) {
        let query = FactQuery::with_check(FactQueryCheck::Match(f_type.clone()));
        if let Some(fact) = self.find_fact_mut(query) {
            fact.confidence = confidence;
            fact.expiration = expiration;
            return;
        }
        self.add_working_memory_fact(f_type, confidence, expiration);
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

    /// marks facts as invalid & cleans the list
    pub fn validate(&mut self) {
        self.to_remove
            .extend(self.facts_list.iter_mut().enumerate().filter_map(|(i, f)| {
                if f.is_valid && (f.update_time.elapsed().unwrap().as_secs_f64() > f.expiration) {
                    f.is_valid = false;
                    return Some(i);
                }
                None
            }));
        if self.facts_list.len() >= self.clean_threshold && self.to_remove.len() > 1 {
            self.clean();
        }
    }

    pub fn clean(&mut self) {
        self.to_remove.clear();
        self.facts_list.retain(|f| f.is_valid);
    }

    fn facts(&self) -> impl Iterator<Item = &WorkingMemoryFact> {
        self.facts_list.iter().filter(|f| f.is_valid)
    }

    fn facts_mut(&mut self) -> impl Iterator<Item = &mut WorkingMemoryFact> {
        self.facts_list.iter_mut().filter(|f| f.is_valid)
    }

    pub fn find_fact(&self, query: FactQuery) -> Option<&WorkingMemoryFact> {
        self.facts().find(|&fact| fact.matches_query(&query))
    }

    pub fn find_fact_mut(&mut self, query: FactQuery) -> Option<&mut WorkingMemoryFact> {
        self.facts_mut().find(|fact| fact.matches_query(&query))
    }

    pub fn find_fact_with_max_confidence(
        &self,
        fact_query: FactQuery,
    ) -> Option<&WorkingMemoryFact> {
        let (fact, _max_confidence) = self.facts().filter(|f| f.matches_query(&fact_query)).fold(
            (None, 0.0),
            |(max_f, max_c), other_f| {
                if max_f.is_none() {
                    return (Some(other_f), other_f.confidence);
                }
                if other_f.confidence > max_c {
                    return (Some(other_f), other_f.confidence);
                }
                (max_f, max_c)
            },
        );
        fact
    }

    pub fn mark_as_invalid(&mut self, query: FactQuery) {
        let Some(fact_index) = self
            .facts_list
            .iter()
            .position(|fact| fact.matches_query(&query))
        else {
            return;
        };
        self.to_remove.push_back(fact_index);
        let fact = &mut self.facts_list[fact_index];
        fact.is_valid = false;
    }

    /// marks given fact as invalid and returns mutable reference
    pub fn find_and_mark_as_invalid(&mut self, query: FactQuery) -> Option<&mut WorkingMemoryFact> {
        let fact_index = self
            .facts_list
            .iter()
            .position(|fact| fact.matches_query(&query))?;
        self.to_remove.push_back(fact_index);
        let fact = &mut self.facts_list[fact_index];
        fact.is_valid = false;
        Some(fact)
    }
}

#[derive(Debug, Clone)]
pub enum FactQueryCheck {
    Match(WMProperty),
    AIStimuli(WMAIStimuliType),
    Node(WMNodeType),
    TaskType(WMTaskType),
    Knowledge(WMKnowledgeType),
    Desire(WMDesireType),
    Event(WMEventType),
}

#[derive(Default, Debug, Clone)]
pub struct FactQuery {
    pub checks: Vec<FactQueryCheck>,
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
