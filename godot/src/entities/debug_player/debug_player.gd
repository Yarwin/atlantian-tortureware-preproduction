extends CharacterBody3D
@export var speed := 4.5


func _physics_process(delta: float) -> void:
	var direction = Vector3.ZERO
	var move_input = Vector2(
		Input.get_action_strength("move_right") - Input.get_action_strength("move_left"),
		Input.get_action_strength("move_forward") - Input.get_action_strength("move_back")
	).normalized()
	direction += (global_transform.basis.x * move_input.x - global_transform.basis.z * move_input.y)
	direction = direction.normalized()
	velocity = speed * direction
	move_and_slide()
