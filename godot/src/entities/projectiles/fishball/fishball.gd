extends CharacterBody3D

@export var speed := 64.5
@export var act_react: ActReactResource
var caster: Object

func _physics_process(delta: float) -> void:
	velocity = transform.basis.z * speed * delta
	var collision = move_and_collide(velocity)
	if collision:
		var collider = collision.get_collider(0)
		if collider != caster:
			react_and_vanish(collider)


func react_and_vanish(collider):
	if collider.get("act_react"):
		GameSystems.act_react_executor.react(self.act_react, collider.get("act_react"), {})
		GameSystems.act_react_executor.react(collider.get("act_react"), self.act_react, {})
	queue_free()
