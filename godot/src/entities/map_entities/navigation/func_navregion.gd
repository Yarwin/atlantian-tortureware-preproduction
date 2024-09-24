@tool
extends NavigationRegion3D
class_name MapNavigationRegion

@export var layers_disabled: int
# initial navigation layers enabled on runtime
@export var layers_enabled: int

@export var behavior: FuncUtils.BehaviourMode = FuncUtils.BehaviourMode.DOOR
@export var adapter: FuncNavigationConnectionAdapter

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

func _on_mover_movement_started(is_reverse: bool):
	if self.adapter:
		self.adapter._on_mover_movement_started(self, is_reverse)

func _on_mover_movement_finished(is_reverse: bool):
	if self.adapter:
		self.adapter._on_mover_movement_finished(self, is_reverse)
