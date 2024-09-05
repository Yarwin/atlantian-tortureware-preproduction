@tool
extends AnimatableBody3D

@export var amount: float
@export var axis: Vector3
var moved: float = 0.0
var moving: bool = false
@export var should_reverse: bool = false
@export var cyclic: bool = false
@export var reverse_delay: float = 0.0
@export var delay: float = 0.0
@export var travel_time: float = 2.0
@export var reverse_travel_time: float = 2.0
var tween
@onready var target_pos: Vector3 = self.position
@onready var starter_pos: Vector3 = self.position

func _func_godot_apply_properties(entity_properties: Dictionary):
	sync_to_physics = false
	var target_name = entity_properties.get("_target_name")
	if target_name:
		add_to_group(StringName(target_name), true)
		notify_property_list_changed()
	amount = entity_properties.get("_amount", 0)
	axis = entity_properties.get("axis", Vector3.ZERO).normalized()
	should_reverse = entity_properties.get("_reverse", false)
	cyclic = entity_properties.get("_cyclic", false)
	delay = entity_properties.get("_delay", 0.0)
	reverse_delay = entity_properties.get("_reverse_delay", delay)
	travel_time = entity_properties.get("_travel_time", 2.0)
	reverse_travel_time = entity_properties.get("_reverse_travel_time", travel_time)
	var parent_name = entity_properties.get("_parent_name")
	if parent_name:
		name = parent_name
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


func move(target: float, is_reverse: bool = false):
	if moving: return
	if tween:
		tween.kill()
	tween = create_tween()
	moving = true
	var current_travel_time = travel_time
	if is_reverse:
		current_travel_time = reverse_travel_time

	var property_tweener = ((tween as Tween)
	.tween_property(self, "target_pos", self.position + target * axis, current_travel_time)
	.set_trans(Tween.TRANS_CUBIC)
	)
	if !is_zero_approx(delay):
		property_tweener.set_delay(delay)
	(tween as Tween).tween_callback(deactivate).set_delay(0.01)
	
	if !(should_reverse and !is_reverse):
		return
	
	property_tweener = ((tween as Tween)
	.tween_callback(move.bind(-target, true)))
	
	if reverse_delay:
		property_tweener.set_delay(reverse_delay)

func deactivate():
	moving = false

func activate():
	if is_zero_approx((starter_pos - target_pos).length_squared()):
		self.move(self.amount)
	else:
		self.move(-self.amount)

func _physics_process(delta: float) -> void:
	if moving:
		self.position = target_pos
	return
