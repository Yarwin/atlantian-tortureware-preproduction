mod godot_api;
pub mod ai;
mod actions;
mod goals;
mod thinker_states;
mod sensors;
mod targeting;
mod ai_nodes;
mod animations;
mod godot_thinker_components;

use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}
