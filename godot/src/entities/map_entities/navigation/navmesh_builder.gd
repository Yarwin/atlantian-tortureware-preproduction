@tool
extends Node3D

class_name NavigationRegionBuilder

# a collection of very retarded sets
@onready var navmesh_groups := {}
@onready var func_movers_navmesh_groups := {}

var source_geometry: NavigationMeshSourceGeometryData3D

###############################################################################
# Builtin functions                                                           #
###############################################################################

###############################################################################
# Private functions                                                           #
###############################################################################

static func _create_default_navigation_mesh() -> NavigationMesh:
	var navigation_mesh = NavigationMesh.new()
	navigation_mesh.geometry_parsed_geometry_type = NavigationMesh.PARSED_GEOMETRY_STATIC_COLLIDERS
	navigation_mesh.agent_radius = 0.5
	navigation_mesh.agent_max_slope = 60
	navigation_mesh.geometry_collision_mask = 1
	return navigation_mesh

func _bake_group(group_name: String, navigation_mesh: NavigationMesh, navigation_region: MapNavigationRegion):
	navigation_mesh.geometry_source_geometry_mode = NavigationMesh.SOURCE_GEOMETRY_GROUPS_WITH_CHILDREN
	navigation_mesh.geometry_source_group_name = group_name
	navigation_region.bake_navigation_mesh.call_deferred()

func _bake_bounded(bounds: FuncNavregionBounds, navigation_mesh: NavigationMesh, navigation_region: MapNavigationRegion):
	navigation_mesh.border_size = bounds.growth
	var source_geo = NavigationMeshSourceGeometryData3D.new()
	NavigationServer3D.parse_source_geometry_data(navigation_mesh, source_geo, %FuncGodotMap)
	var r_bounds = bounds.get_bounds()
	if r_bounds:
		navigation_mesh.filter_baking_aabb = r_bounds.grow(bounds.growth)
		navigation_mesh.filter_baking_aabb_offset = navigation_region.to_local(bounds.get_global_position())
	else:
		print("couldn't get bounds for given navigation region! ", bounds)
		assert(false == true)
		return
	for obstacle in (get_tree().get_nodes_in_group("navigation_baking_obstacles") as Array[NavigationBakingObstacle3D]):
		obstacle.add_obstructions_to_geometry(source_geo)
	NavigationServer3D.bake_from_source_geometry_data(navigation_mesh, source_geo)
	navigation_region.layers_enabled = bounds.layers_enabled
	navigation_region.layers_disabled = bounds.layers_disabled
	navigation_region.use_edge_connections = bounds.use_edge_connections
	if bounds.is_enabled:
		navigation_region.navigation_layers = navigation_region.layers_enabled
	else:
		navigation_region.navigation_layers = navigation_region.layers_disabled
	if bounds.target_names:
		navigation_region.behavior = bounds.behavior
		FuncUtils.connect_entity_to_movers(navigation_region, bounds.target_names)


func _attach_region_to_parent(bounds: FuncNavregionBounds, region: MapNavigationRegion, default: Node3D):
	var parent = default
	if bounds and bounds.parent:
		var declared_parent: Node3D = FuncUtils.get_parent(bounds.parent, bounds)
		if declared_parent:
			parent = declared_parent
	parent.add_child(region)
	region.owner = parent.owner


func _create_navigation_region(new_name: String, navmesh: NavigationMesh) -> MapNavigationRegion:
	var navigation_region_node = MapNavigationRegion.new()
	navigation_region_node.name = new_name
	navigation_region_node.navigation_mesh = navmesh
	return navigation_region_node


func _build_navregion(region_group: String):
	var navigation_mesh = _create_default_navigation_mesh()
	var bounds: FuncNavregionBounds = get_tree().get_first_node_in_group(region_group + "_bounds")
	var navigation_region_node = _create_navigation_region(region_group, navigation_mesh)
	self.add_child(navigation_region_node)
	navigation_region_node.owner = self.owner
	if bounds:
		_bake_bounded(bounds, navigation_mesh, navigation_region_node)
		self.remove_child(navigation_region_node)
		_attach_region_to_parent(bounds, navigation_region_node, self)
		if navigation_region_node.get_parent() != self:
			navigation_region_node.position = navigation_region_node.to_local(self.global_position)
		bounds.queue_free()
	else:
		_bake_group(region_group, navigation_mesh, navigation_region_node)


func _build_navregion_for_func_mover(region_group: String):
	var mover: FuncMover = get_tree().get_first_node_in_group(region_group)
	if not mover: return
	var navigation_mesh = _create_default_navigation_mesh()
	var bounds: FuncNavregionBounds = get_tree().get_first_node_in_group(region_group + "_bounds")
	if bounds:
		var r_bounds = bounds.get_bounds()
		if r_bounds:
			navigation_mesh.filter_baking_aabb = r_bounds
		bounds.queue_free()
	var navigation_region_node = _create_navigation_region(region_group, navigation_mesh)
	_attach_region_to_parent(null, navigation_region_node, mover)
	# disable edge connections – agents should use navigation links instead
	navigation_region_node.use_edge_connections = false
	var source_geo: NavigationMeshSourceGeometryData3D = NavigationMeshSourceGeometryData3D.new()
	NavigationServer3D.parse_source_geometry_data(navigation_mesh, source_geo, mover)
	NavigationServer3D.bake_from_source_geometry_data(navigation_mesh, source_geo)
	FuncUtils.connect_entity_to_mover(navigation_region_node, mover)

func _build_navregions():
	for navmesh_group in self.navmesh_groups.keys():
		_build_navregion(navmesh_group)
	for navmesh_group in self.func_movers_navmesh_groups.keys():
		_build_navregion_for_func_mover(navmesh_group)

###############################################################################
# Public functions                                                            #
###############################################################################

func add_new_navmesh_group(nav_group: String):
	self.navmesh_groups[nav_group] = null

func add_new_func_mover_navmesh_group(nav_group: String):
	self.func_movers_navmesh_groups[nav_group] = null

###############################################################################
# Connections                                                                 #
###############################################################################

func _on_func_godot_map_build_complete() -> void:
	# disable collisions on entities such as doors
	get_tree().call_group("disable_for_baking", "disable_collisions")
	for child in self.get_children():
		child.queue_free()
	await get_tree().process_frame
	self._build_navregions()
	self.navmesh_groups = {}
	self.func_movers_navmesh_groups = {}
	get_tree().call_group("disable_for_baking", "enable_collisions")
