extends FuncNavigationConnectionAdapter

class_name FuncTrapdoorAdapter

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

func _on_mover_movement_started(_entity, _is_reverse: bool):
    return


func _on_mover_movement_finished(entity, is_reverse: bool):
    if (entity.is_enabled and !is_reverse) or (!entity.is_enabled and is_reverse):
        entity.navigation_layers = entity.layers_enabled
    else:
        entity.navigation_layers = entity.layers_disabled
