@tool
extends ActReactArea3D

class_name FuncButton

@export var target_name: StringName

func _func_godot_apply_properties(entity_properties: Dictionary):
	target_name = StringName(entity_properties.get("_target"))


func _ready():
	if Engine.is_editor_hint(): return
	if target_name:
		target = get_tree().get_first_node_in_group(target_name)
