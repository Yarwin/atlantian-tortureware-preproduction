extends CharacterController3D

var is_jumping = false
var is_falling = false
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
		is_jumping = true


	if is_jumping and self.velocity.y > 0:
		is_jumping = false
		is_falling = true
		self.gravity_scale = jump_gravity_scale
	elif is_falling and is_zero_approx(self.velocity.y):
		self.gravity_scale = default_gravity_scale
	process_movement(delta)
