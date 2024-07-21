@tool
extends AnimatableBody3D

@export var move_vec: Vector3
@export var move_length: float

var moved: float = 0.0
var moving: bool = false


func _func_godot_apply_properties(entity_properties: Dictionary):
	var target_name = entity_properties.get("_target_name")
	if target_name:
		add_to_group(StringName(target_name), true)
		notify_property_list_changed()


func move():
	moving = true 


func _physics_process(delta: float) -> void:
	if not moving: return
	self.global_position += move_vec
	self.moved += move_vec.length()
