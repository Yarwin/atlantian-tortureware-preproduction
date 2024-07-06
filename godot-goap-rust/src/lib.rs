mod actions;
pub mod ai;
mod ai_nodes;
mod animations;
mod goals;
mod godot_api;
mod godot_thinker_components;
mod sensors;
mod targeting;
mod thinker_states;

use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
