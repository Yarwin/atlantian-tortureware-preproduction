extends Panel

@onready var text_label := $RichTextLabel


func _ready():
	EventBus.new_debug_entity_picked.connect(_on_new_debug_entity_picked)


func _on_new_debug_entity_picked(info, entity_id):
	var text: String = "[b] Thinker " + str(entity_id) + "[/b]" + "\n"
	text += "[b]Goal[/b] " + info.get("goal") + "\n"
	text += "[b]Action[/b] " + info.get("action") + "\n"
	text += "[b]Current WS[/b] " + info.get("current_world_state")
	text_label.text = text
