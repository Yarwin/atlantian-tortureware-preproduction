extends CharacterController3D


func _physics_process(delta: float) -> void:
	direction = Vector3.ZERO
	var move_input = Input.get_vector(
		"move_left", "move_right", "move_back", "move_forward"
	)
	direction += (basis.x * move_input.x - basis.z * move_input.y)
	direction = direction.normalized()
	process_movement(delta)
