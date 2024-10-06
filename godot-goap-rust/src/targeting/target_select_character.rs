use crate::ai::working_memory::{FactQuery, FactQueryCheck, Knowledge, WMKnowledgeType, WMProperty};
use crate::sensors::sensor_types::ThinkerProcessArgs;
use crate::targeting::target::AITarget;


pub fn select_character(args: &mut ThinkerProcessArgs) -> Option<AITarget> {
    let fact_query = FactQuery::with_check(FactQueryCheck::Knowledge(WMKnowledgeType::Character));
    let fact = args.working_memory.find_fact_with_max_confidence(fact_query)?;
    let WMProperty::Knowledge(Knowledge::Character(character_id, pos)) = fact.f_type else {return None;};
    // todo – create some hitpoint selector
    Some(AITarget::Character(character_id, pos))
}
