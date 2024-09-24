/// attack subsystem is responsible for finding the best attack for given entity
/// and putting this information inside blackboard

use crate::animations::animation_data::AnimationProps;


pub enum AttackValidator {
    VisibleCharacter,
    Range
}


/// Immutable struct that keeps all the immutable data related to given attack
#[derive(Debug)]
pub struct AttackData {
    /// Attack name
    name: String,
    /// Attack range min/max
    range: (f32, f32),
    /// How fast given attack can be executed again.
    /// Information about given attack is being kept in working memory.
    cooldown: f64,
    /// default priority of given attack. Can be edited (for example entity might follow some attack with combo)
    default_weight: f32,
    /// Steps required to perform given attack (prepare/execute/recover)
    steps: Vec<AnimationProps>,
}

fn process_attack_manager() {
    // bail if no target

    // 1. filter attacks leaving only the valid ones

    // 2. choose one attack

    // 3. put information about given attack in the blackboard

}