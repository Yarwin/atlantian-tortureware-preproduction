extends Node3D

@export var hitscan_decal: PackedScene
@export var world_node: Node3D

func _ready() -> void:
	GameSystems.new_hitscan_collision_registered.connect(on_new_hitscan_collision)


func on_new_hitscan_collision(col: Vector3, normal: Vector3):
	var decal = hitscan_decal.instantiate()
	world_node.add_child(decal)
	decal.global_position = col
