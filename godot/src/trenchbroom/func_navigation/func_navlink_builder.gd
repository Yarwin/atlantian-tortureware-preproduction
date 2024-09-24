@tool
extends Marker3D

var is_start: bool
var groups: Array[String] = []
var nav_name: String
var bidirectional := true

###############################################################################
# Builtin functions                                                           #
###############################################################################

func _func_godot_apply_properties(entity_properties: Dictionary):
	nav_name = entity_properties.get("_name")
	is_start = entity_properties.get("_is_start", true)
	# parent given navlink to moving platforms and whatnot
	var parent_name = entity_properties.get("_parent", true)
	if parent_name:
		FuncUtils.reparent_node(parent_name, self)

	if !is_start:
		add_to_group(nav_name)
	else:
		bidirectional = entity_properties.get("_bidirectional", true)
		groups.append_array(entity_properties.get("_target_names", "").split(","))
		_create_navigation_link.call_deferred()

###############################################################################
# Private functions                                                           #
###############################################################################

func _create_navigation_link():
	self.rotation = Vector3.ZERO
	var nav_end: Marker3D = get_tree().get_first_node_in_group(nav_name)
	if not nav_end:
		queue_free()
		return
	var end_pos = self.to_local(nav_end.global_position)
	var navlink = FuncNavLink3D.new()
	self.get_parent().add_child(navlink)
	navlink.owner = self.owner
	navlink.bidirectional = self.bidirectional
	navlink.global_position = self.global_position
	navlink.end_position = end_pos
	navlink.name = self.nav_name + "_link"
	for group in self.groups:
		if not group: continue
		navlink.add_to_group(group, true)
	nav_end.queue_free()
	queue_free()

###############################################################################
# Public functions                                                            #
###############################################################################


###############################################################################
# Connections                                                                 #
###############################################################################
