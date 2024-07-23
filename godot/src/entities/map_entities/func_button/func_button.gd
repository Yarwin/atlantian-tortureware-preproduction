@tool
extends ActReactArea3D

class_name FuncButton

@export var target_name: StringName

func _func_godot_apply_properties(entity_properties: Dictionary):
	target_name = StringName(entity_properties.get("_target"))


func _post_ready():
	if Engine.is_editor_hint(): return
	if target_name:
		var potential_target: Node3D = get_tree().get_first_node_in_group(target_name) as Node3D
		target = potential_target
		#print(potential_target)
		#set_deferred("target", potential_target)
