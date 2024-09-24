@tool
extends Object
class_name FuncUtils

enum BehaviourMode { ## Enum specifing behaviour of given navigation entity in relation to FuncMover.
    DOOR = 0, ## disables entity when given mover starts movement; enables back after reverse movement is finished
    LIFT = 1, ## always disables given entity while movement starts and enables it back when it ends
    TRAPDOOR = 2, ## enables/disables given entity when movement ends
}

###############################################################################
# Builtin functions                                                           #
###############################################################################


###############################################################################
# Private functions                                                           #
###############################################################################


###############################################################################
# Public functions                                                            #
###############################################################################


static func get_parent(parent_name: String, node: Node):
    if node.get_parent().name == parent_name:
        return node.get_parent()
    if !node.is_inside_tree(): return;
    if !node.get_parent().has_node(parent_name): return;
    return node.get_parent().get_node(NodePath(parent_name))

static func reparent_node(parent_name: String, node: Node):
    if node.get_parent().name == parent_name:
        return
    if !node.is_inside_tree(): return;

    if !node.get_parent().has_node(parent_name): return;
    var t: Transform3D = node.global_transform
    var new_parent: Node = node.get_parent().get_node(NodePath(parent_name))
    node.get_parent().remove_child(node)
    new_parent.add_child(node)
    node.global_transform = t
    node.owner = new_parent.owner
    for child in node.get_children():
        child.owner = node.owner

static func connect_entity_to_mover(entity: Node, mover: FuncMover):
    match entity.behavior:
        BehaviourMode.DOOR:
            entity.adapter = load("res://src/entities/map_entities/navigation/adapters/func_door_adapter.tres")
        BehaviourMode.LIFT:
            print("lift")
        BehaviourMode.TRAPDOOR:
            entity.adapter = load("res://src/entities/map_entities/navigation/adapters/func_trapdoor_adapter.tres")
    if not entity.adapter: return
    mover.movement_started.connect(entity._on_mover_movement_started, CONNECT_PERSIST)
    mover.movement_finished.connect(entity._on_mover_movement_finished, CONNECT_PERSIST)
    entity.notify_property_list_changed()

static func connect_entity_to_movers(entity: Node, groups: PackedStringArray):
    if groups.size() == 0: return;
    for group_name in groups:
        var targets = entity.get_tree().get_nodes_in_group(group_name)
        for target in targets:
            if target is not FuncMover:
                print("expected FuncMover in given group for ", entity, "found ", target)
            connect_entity_to_mover(entity, target)

###############################################################################
# Connections                                                                 #
###############################################################################
