mod goap_actions;
pub mod ai;
mod ai_nodes;
mod animations;
mod goap_goals;
mod godot_api;
mod godot_thinker_components;
mod sensors;
mod targeting;
mod thinker_states;
mod character_controler;
mod player_controller;
pub mod godot_api_acts;
pub mod godot_api_reacts;
pub mod act_react;
mod godot_entities;

use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
