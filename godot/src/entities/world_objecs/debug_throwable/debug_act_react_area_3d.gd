extends ActReactArea3D

@export var while_grabbed_act_react: ActReactResource
@onready var normal_act_react: ActReactResource = self.act_react



func _on_debug_world_object_grabbed() -> void:
	if while_grabbed_act_react:
		set_act_react(while_grabbed_act_react)


func _on_debug_world_object_released() -> void:
	set_act_react(normal_act_react)


func _on_debug_world_object_contact_velocity_achieved() -> void:
	var vel_effects = get_parent().contact_velocity_effects
	if vel_effects:
		set_act_react(get_parent().contact_velocity_effects)

func _on_debug_world_object_contact_velocity_left() -> void:
	set_act_react(normal_act_react)
