@tool
extends Node3D

class_name NavigationBakingObstacle3D

var outlines: Array[PackedVector3Array] = []
var height: float = 1.0
var elevation: float  

func _func_godot_apply_properties(_entity_properties: Dictionary):
	calculate_outlines()
	add_to_group("navigation_baking_obstacles")


func calculate_outlines():
	for child in self.get_children():
		if not (child is MeshInstance3D):
			continue
		calculate_outline(child)


func calculate_outline(instance: MeshInstance3D):
	var mesh_data_tool = MeshDataTool.new()
	height = instance.mesh.get_aabb().size.y
	elevation = instance.global_position.y - 0.5 * height
	mesh_data_tool.create_from_surface((instance as MeshInstance3D).mesh, 0)
	
	for e_idx in range(mesh_data_tool.get_edge_count()):
		var quads_set: Dictionary = {}
		for f_idx in mesh_data_tool.get_edge_faces(e_idx):
			var face_normal = mesh_data_tool.get_face_normal(f_idx)
			if face_normal != Vector3.DOWN:
				break
			for v_idx in range(0, 3):
				quads_set[mesh_data_tool.get_face_vertex(f_idx, v_idx)] = null
		var unwrapped_set = quads_set.keys()
		if len(unwrapped_set) == 4:
			outlines.append(PackedVector3Array([
				Vector3(1, 0, 1) * (instance.global_position + mesh_data_tool.get_vertex(unwrapped_set[0])),
				Vector3(1, 0, 1) * (instance.global_position + mesh_data_tool.get_vertex(unwrapped_set[1])),
				Vector3(1, 0, 1) * (instance.global_position + mesh_data_tool.get_vertex(unwrapped_set[2])),
				Vector3(1, 0, 1) * (instance.global_position + mesh_data_tool.get_vertex(unwrapped_set[3])),
			]))

			

func add_obstructions_to_geometry(geometry: NavigationMeshSourceGeometryData3D):
	# for child in self.get_children():
	# 	if not (child is MeshInstance3D):
	# 		continue
	# 	var mesh_data_tool = MeshDataTool.new()
	# 	var outlines: Array[PackedVector3Array] = []
	# 	var aabb = (child as MeshInstance3D).mesh.get_aabb()
	# 	mesh_data_tool.create_from_surface((child as MeshInstance3D).mesh, 0)

	# 	for e_idx in range(mesh_data_tool.get_edge_count()):
	# 		var quad: Dictionary = {}
	# 		for f_idx in mesh_data_tool.get_edge_faces(e_idx):
	# 			if mesh_data_tool.get_face_normal(f_idx) != Vector3.DOWN:
	# 				break
	# 			for v_idx in range(0, 3):
	# 				quad[mesh_data_tool.get_face_vertex(f_idx, v_idx)] = null
	# 		var unwrapped_set = quad.keys()
	# 		if len(unwrapped_set) == 4:
	# 			outlines.append(PackedVector3Array([
	# 				Vector3(1, 0, 1) * (child.global_position + mesh_data_tool.get_vertex(unwrapped_set[0])),
	# 				Vector3(1, 0, 1) * (child.global_position + mesh_data_tool.get_vertex(unwrapped_set[1])),
	# 				Vector3(1, 0, 1) * (child.global_position + mesh_data_tool.get_vertex(unwrapped_set[2])),
	# 				Vector3(1, 0, 1) * (child.global_position + mesh_data_tool.get_vertex(unwrapped_set[3])),
	# 			]))
	for outline in outlines:
		geometry.add_projected_obstruction(outline, elevation, height, false)
	queue_free()

# func get_obstacles_outline() -> Array[PackedVector3Array]:
#     var outline = []
#     for child in self.get_children():
#         if not (child is MeshInstance3D):
#             continue
#         var aabb = (child as MeshInstance3D).mesh.get_aabb()
#         outline.append(
#             PackedVector3Array(
#                 [
#             aabb.size / 2.0 * Vector3(1, 0, 1),
#             aabb.size / 2.0 * Vector3(-1, 0, 1),
#             aabb.size / 2.0 * Vector3(-1, 0, -1),
#             aabb.size / 2.0 * Vector3(1, 0, -1),
#         ]
#             )
#         )
#     return outline

# func get_obstacle_elevation():
#     pass

# func get_obstacle_height():
#     pass

# func get_obstacle_carve_mesh():
#     pass
