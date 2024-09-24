@tool
extends Area3D

class_name FuncNavregionBounds

var layers_disabled = 1
var layers_enabled: int = 1
var target_names: PackedStringArray
var growth: float = 1.0
var is_enabled: bool = true
var behavior: FuncUtils.BehaviourMode = FuncUtils.BehaviourMode.DOOR
var use_edge_connections: bool = true
var parent: String

func _func_godot_apply_properties(entity_properties: Dictionary):
	var navregion_group = entity_properties.get("_navregions")
	add_to_group(navregion_group + "_bounds", true)
	layers_disabled = entity_properties.get("_layers_disabled", 1)
	layers_enabled = entity_properties.get("_layers_enabled", 1)
	is_enabled = entity_properties.get("_is_enabled", true)
	growth = entity_properties.get("_grow", 1.0)
	parent = entity_properties.get("_parent")
	behavior = entity_properties.get("_behavior", FuncUtils.BehaviourMode.DOOR)
	use_edge_connections = entity_properties.get("_use_edge_connections", true)
	target_names = entity_properties.get("_target_names", "").split(",")
	get_tree().call_group("navmesh_builder", "add_new_navmesh_group", navregion_group)


func get_bounds() -> AABB:
	var shape: RID = PhysicsServer3D.area_get_shape(get_rid(), 0)
	var s_type = PhysicsServer3D.shape_get_type(shape)
	var bounds = AABB()
	if s_type == PhysicsServer3D.SHAPE_CONVEX_POLYGON:
		var s_data: PackedVector3Array = PhysicsServer3D.shape_get_data(shape)
		var points_count = s_data.size()
		bounds.position = s_data[0]
		for i in range(1, points_count):
			bounds = bounds.expand(s_data[i])
	return bounds
