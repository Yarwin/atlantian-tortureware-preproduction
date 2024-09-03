extends Control

var slide_tween: Tween
var current = 0
@onready var child_count := get_child_count()
signal label_slided

###############################################################################
# Builtin functions                                                           #
###############################################################################

func _ready() -> void:
	GameSystems.new_log_message.connect(_on_new_log)


###############################################################################
# Private functions                                                           #
###############################################################################

func hide_child_and_slide(to_hide: Control):

	if slide_tween:
		if slide_tween.is_running():
			await slide_tween.finished
		slide_tween.kill()
	
	slide_tween = create_tween().set_parallel(true)
	var child_size = to_hide.get_size().y
	slide_tween.tween_property(to_hide, "modulate:a", 0.0, 0.5).set_trans(Tween.TRANS_EXPO)
	for child in get_children():
		slide_tween.tween_property(child, "position:y", child.position.y - child_size, 0.5).from_current().set_trans(Tween.TRANS_EXPO)
	await slide_tween.finished
	to_hide.hide()
	label_slided.emit()

func get_first_free_label():
	var label_idx = current + 1

	while label_idx != current:
		if label_idx >= child_count - 1:
			label_idx = 0
		else:
			label_idx += 1
		var potential_child = get_child(label_idx)
		if !potential_child.visible:
			current = label_idx
			return potential_child


###############################################################################
# Public functions                                                            #
###############################################################################


###############################################################################
# Connections                                                                 #
###############################################################################

func _on_new_log(log: String):
	if slide_tween and slide_tween.is_running():
		await slide_tween.finished
		
	var label = null
	
	while true:
		label = get_first_free_label()
		if not label:
			await label_slided
		else:
			break
	
	label.text = log
	label.modulate.a = 0.0

	var offset_y = 0
	for child in get_children():
		if child == label: continue
		if !child.visible: continue
		var offset_child_y = child.position.y + child.get_size().y
		if offset_child_y > offset_y:
			offset_y = offset_child_y
	label.position.y = offset_y
	label.show()
	var tween = label.create_tween()
	tween.tween_property(label, "modulate:a", 1.0, 0.5).set_trans(Tween.TRANS_EXPO)
	tween.tween_callback(hide_child_and_slide.bind(label)).set_delay(1.0)
