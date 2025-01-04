pub mod act_react;
pub mod ai;
mod ai_attacks;
mod ai_nodes;
mod animations;
mod character_controler;
mod equipment;
mod goap_actions;
mod goap_goals;
mod godot_api;
pub mod godot_api_acts;
pub mod godot_api_reacts;
mod godot_entities;
mod inventory;
mod inventory_ui;
mod multi_function_display;
mod player_controller;
mod receiver;
mod sensors;
mod targeting;
mod thinker_states;
mod utils;

use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
