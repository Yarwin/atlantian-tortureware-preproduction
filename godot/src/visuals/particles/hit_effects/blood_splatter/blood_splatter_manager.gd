extends Node3D

@export var amount := 1
var blood_scene = preload("res://src/visuals/particles/hit_effects/blood_splatter/blood_splatter.tscn")
var current = 0

func _ready() -> void:
	for _i in range(amount):
		var splatter = blood_scene.instantiate()
		self.add_child(splatter)


func activate_splatter(pos: Vector3, normal: Vector3):
	var initial = current;
	var splatter_idx = initial + 1
	var splatter = null
	while splatter_idx != initial: 
		var potential_splatter: GPUParticles3D = get_child(splatter_idx)
		if !potential_splatter.emitting:
			splatter = potential_splatter
			break
		splatter_idx += 1
		if splatter_idx > amount:
			splatter_idx = 0
	if not splatter:
		return

	splatter.global_position = pos
	splatter.global_rotation = normal
	splatter.restart()


func _on_damage_taken(_damage: float, pos: Vector3, normal: Vector3):
	self.activate_splatter(pos, normal)
	
