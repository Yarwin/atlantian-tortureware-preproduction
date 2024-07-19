extends Area3D
@onready var think: Thinker = $"../thinker"

func _on_mouse_entered() -> void:
	EventBus.new_debug_entity_picked.emit(AiManager.get_thinker_debug_data(think.thinker_id), think.thinker_id)

func _on_mouse_exited() -> void:
	pass
	
