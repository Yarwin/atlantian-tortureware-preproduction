extends CharacterController3D


@onready var default_gravity_scale = self.gravity_scale
@export var jump_gravity_scale: float

func _physics_process(delta: float) -> void:
	direction = Vector3.ZERO
	var move_input = Input.get_vector(
		"move_left", "move_right", "move_back", "move_forward"
	)
	direction += (basis.x * move_input.x - basis.z * move_input.y)
	if !direction.is_zero_approx():
		direction = direction.normalized()

	if Input.is_action_just_pressed("jump"):
		direction.y += 1.0
	if velocity.y <= 0:
		self.gravity_scale = jump_gravity_scale
	else:
		self.gravity_scale = default_gravity_scale

	process_movement(delta)
