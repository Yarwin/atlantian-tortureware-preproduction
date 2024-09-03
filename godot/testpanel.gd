extends Panel

@onready var text_label := $RichTextLabel


func _ready():
	GameSystems.new_debug_info.connect(_on_new_debug_info)
	return
	#EventBus.new_debug_entity_picked.connect(_on_new_debug_entity_picked)


func _on_new_debug_info(info):
	text_label.text = info
