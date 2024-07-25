@tool
extends AnimatableBody3D

@export var move_vec: Vector3
@export var amount: float
@export var axis: Vector3

var moved: float = 0.0
var moving: bool = false


func _func_godot_apply_properties(entity_properties: Dictionary):
	var target_name = entity_properties.get("_target_name")
	if target_name:
		add_to_group(StringName(target_name), true)
		notify_property_list_changed()
	amount = entity_properties.get("_amount")
	axis = entity_properties.get("axis")


func move():
	moving = true 


func _physics_process(delta: float) -> void:
	if not moving: return
	self.global_position += axis * delta
	self.moved += (axis * 0.1).length()
	if self.moved > self.amount:
		moving = false
