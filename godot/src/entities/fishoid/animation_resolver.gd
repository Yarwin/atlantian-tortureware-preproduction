extends Node
@export var body: CharacterBody3D
@export var projectile_scene: PackedScene
@export var muzzle: Marker3D
@export var thinker: Thinker

func shoot():
	var projectile: Node3D = projectile_scene.instantiate()
	add_child(projectile)
	projectile.top_level = true
	
	var target = thinker.get_target()
	if target:
		projectile.global_transform = body.global_transform.looking_at(target, Vector3(0, 1, 0), true)
	else:
		print("no target")
	projectile.global_position = muzzle.global_position
	projectile.caster = owner
	
