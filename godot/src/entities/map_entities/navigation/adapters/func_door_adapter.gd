extends FuncNavigationConnectionAdapter

class_name FuncDoorAdapter

###############################################################################
# Builtin functions                                                           #
###############################################################################


###############################################################################
# Private functions                                                           #
###############################################################################


###############################################################################
# Public functions                                                            #
###############################################################################


###############################################################################
# Connections                                                                 #
###############################################################################

func _on_mover_movement_started(entity, is_reverse: bool):
    if ((entity.is_enabled and is_reverse) or 
        (!entity.is_enabled and !is_reverse)): return
    entity.navigation_layers = entity.layers_disabled


func _on_mover_movement_finished(entity, is_reverse: bool):
    if ((entity.is_enabled and !is_reverse) or 
    (!entity.is_enabled and is_reverse)): return
    entity.navigation_layers = entity.layers_enabled
