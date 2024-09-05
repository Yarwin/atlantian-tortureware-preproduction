@tool
extends ActReactArea3D
@export var target_group: String
@export var one_shot: bool = true
@export var delay := 0.0
@export var recover_time := 0.0
@export var default_layers: int
@export var default_masks: int
var mode: Dictionary = {
	0: preload("res://src/entities/map_entities/func_area/pressure_act_react.tres"),
	1: preload("res://src/entities/map_entities/func_area/button_act_react.tres")
}

func _func_godot_apply_properties(entity_properties: Dictionary):
	self.monitorable = entity_properties.get("_monitorable", true)
	self.name_display = entity_properties.get("_name_display", "")
	if self.monitorable:
		default_layers = self.collision_layer
	else:
		self.collision_layer = 0
	self.monitoring = entity_properties.get("_monitoring", false)
	if self.monitoring:
		default_masks = self.collision_mask
		self.propagation_mode = 1
	else:
		self.collision_mask = 0

	var mode_type: int = entity_properties.get("_mode", 0)
	
	self.act_react = mode[mode_type]
	self.target_group = entity_properties.get("_target")
	self.one_shot = entity_properties.get("_one_shot", false)
	self.delay = entity_properties.get("_delay", 0.0)
	self.recover_time = entity_properties.get("_recover_time", 0.0)
	var parent = entity_properties.get("_parent")
	if parent:
		reparent_to.bind(parent).call_deferred()

func reparent_to(parent_name: String):
	if get_parent().name == parent_name:
		return
	if is_inside_tree():
		if get_parent().has_node(parent_name):
			var t: Transform3D = global_transform
			var new_parent: Node = get_parent().get_node(NodePath(parent_name))
			get_parent().remove_child(self)
			new_parent.add_child(self)
			global_transform = t
			owner = new_parent.owner
			for child in get_children():
				child.owner = owner

func activate():
	self.monitoring = false
	if not target_group: return
	if !is_zero_approx(self.delay):
		await get_tree().create_timer(self.delay).timeout
	get_tree().call_group(target_group, "activate")
	if is_zero_approx(recover_time):
		self.monitoring = !self.one_shot
		return
	if !self.one_shot:
		get_tree().create_timer(recover_time).timeout.connect(recover, CONNECT_ONE_SHOT)


func recover():
	self.monitoring = true
