extends RayCast3D
@onready var player := $"../../.."


func _ready() -> void:
	self.add_exception_rid(player.get_rid())

func check_for_entity():
	self.force_raycast_update()
	var collider = self.get_collider()
	if collider:
		if not collider is CharacterController3D:
			return
		
		if not collider.has_node("thinker"): return
		var thinker: Thinker = collider.get_node("thinker")
		var ai_manager = Engine.get_singleton("AIManager")
		var info = ai_manager.get_thinker_debug_data(thinker.thinker_id)
		var text: String = "[b] Thinker " + str(thinker.thinker_id) + "[/b]" + "\n"
		text += "[b]Goal[/b] " + info.get("goal") + "\n"
		text += "[b]Action[/b] " + info.get("action") + "\n"
		text += "[b]Current WS[/b] " + info.get("current_world_state")
		GameSystems.new_debug_info.emit(text)
