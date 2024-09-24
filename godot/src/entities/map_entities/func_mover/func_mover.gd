@tool
extends AnimatableBody3D

class_name FuncMover

# target position set by tween and synced in physics_process
@onready var target_pos: Vector3 = self.position
@onready var starter_pos: Vector3 = self.position

@export var amount: float
@export var axis: Vector3
@export var should_reverse: bool = false
@export var cyclic: bool = false
@export var reverse_delay: float = 0.0
@export var delay: float = 0.0
@export var travel_time: float = 2.0
@export var reverse_travel_time: float = 2.0
var moving: bool = false
var tween
var region_rid: RID
var default_layers: int

signal movement_finished(is_reverse: bool)
signal movement_started(is_reverse: bool)


###############################################################################
# Builtin functions                                                           #
###############################################################################


func _func_godot_apply_properties(entity_properties: Dictionary):
	sync_to_physics = false
	amount = entity_properties.get("_amount", 0)
	axis = entity_properties.get("axis", Vector3.ZERO).normalized()
	should_reverse = entity_properties.get("_reverse", false)
	cyclic = entity_properties.get("_cyclic", false)
	delay = entity_properties.get("_delay", 0.0)
	reverse_delay = entity_properties.get("_reverse_delay", delay)
	travel_time = entity_properties.get("_travel_time", 2.0)
	reverse_travel_time = entity_properties.get("_reverse_travel_time", travel_time)

	var target_names = entity_properties.get("_target_names")
	if target_names:
		var targets = target_names.split(",")
		for t in targets:
			if not t:  continue
			add_to_group(StringName(t), true)
			notify_property_list_changed()

	var parent_name = entity_properties.get("_parent_name")
	if parent_name:
		name = parent_name
	var parent = entity_properties.get("_parent")
	if parent:
		FuncUtils.reparent_node.bind(parent, self).call_deferred()
	var navregion_groups = entity_properties.get("_navregion")

	if navregion_groups:
		var navregions: PackedStringArray = (navregion_groups as String).split(",")
		for navreg in navregions:
			if not navreg: continue
			add_to_group(navreg, true)
			get_tree().call_group("navmesh_builder", "add_new_func_mover_navmesh_group", navreg)

	if entity_properties.get("_disable_collision_for_baking"):
		self.add_to_group("disable_for_baking", true)
		default_layers = self.collision_layer


func _physics_process(_delta: float) -> void:
	if Engine.is_editor_hint(): return
	if moving:
		self.position = target_pos


###############################################################################
# Private functions                                                           #
###############################################################################


func _reparent_to(parent_name: String):
	FuncUtils.reparent_node(parent_name, self)

###############################################################################
# Public functions                                                            #
###############################################################################


func disable_collisions():
	self.collision_layer = 0


func enable_collisions():
	self.collision_layer = default_layers


func move(target: float, is_reverse: bool = false):
	if moving: return
	if tween:
		tween.kill()
	tween = create_tween()
	moving = true

	var current_travel_time = travel_time
	if is_reverse:
		current_travel_time = reverse_travel_time
	var prop_tweener = (tween as Tween).tween_callback(movement_started.emit.bind(is_reverse))
	if !is_zero_approx(delay):
		prop_tweener.set_delay(delay)
	((tween as Tween)
	.tween_property(self, "target_pos", self.position + target * axis, current_travel_time)
	.set_trans(Tween.TRANS_CUBIC)
	)

	(tween as Tween).tween_callback(deactivate).set_delay(0.01)
	
	if !(should_reverse and !is_reverse):
		return
	
	var property_tweener = ((tween as Tween)
	.tween_callback(move.bind(-target, true)))
	
	if reverse_delay:
		property_tweener.set_delay(reverse_delay)


###############################################################################
# Connections                                                                 #
###############################################################################


func activate():
	if (starter_pos - target_pos).is_zero_approx():
		self.move(self.amount)
	else:
		self.move(-self.amount, true)


func deactivate():
	moving = false
	var is_reverse = !(starter_pos - target_pos).is_zero_approx()
	movement_finished.emit(is_reverse)
