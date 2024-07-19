extends Node
@export var projectile_scene: PackedScene
@export var muzzle: Marker3D
@export var thinker: Thinker

func shoot():
	var projectile: Node3D = projectile_scene.instantiate()
	add_child(projectile)
	projectile.top_level = true
	projectile.global_transform = muzzle.global_transform
	var target = thinker.get_target()
	if target:
		projectile.global_transform = projectile.global_transform.looking_at(target + Vector3(0, 1.2, 0), Vector3(0, 1, 0), true)
	projectile.caster = owner
	
