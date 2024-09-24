extends Control
@onready var frob_info := $VBoxContainer/HBoxContainer/HBoxContainer/FrobInfo
@onready var use_label := $VBoxContainer/TextureRect2/UseLabel
@onready var progress_rect := $VBoxContainer/TextureRect2
@onready var name_display := $VBoxContainer/HBoxContainer/Label
@onready var container := $VBoxContainer/HBoxContainer
var tween

func _ready() -> void:
	self.hide()
	GameSystems.frob_prompt_updated.connect(_on_frob_prompt_updated)
	GameSystems.frob_description_deactivated.connect(_on_frob_description_deactivated)
	GameSystems.frob_progress_updated.connect(_on_frob_progress_updated)
	

func _on_frob_prompt_updated(new: String, progress: float, display: String):
	self.progress_rect.material.set_shader_parameter("value", 0.0)
	progress_rect.custom_minimum_size = Vector2(container.size.y, container.size.y)
	self.show()
	frob_info.text = new
	name_display.text = display
	var use_label_size = use_label.get_rect().size
	if tween:
		tween.kill()
	tween = create_tween()
	tween.set_loops()
	var callable = func(v): self.progress_rect.material.set_shader_parameter("value", v)
	(tween as Tween).tween_method(callable, progress, progress + 0.1, 1.).set_trans(Tween.TRANS_CUBIC)
	(tween as Tween).tween_method(callable, progress + 0.1, progress, 1.).set_trans(Tween.TRANS_CUBIC)


func _on_frob_progress_updated(progress: float):
	if tween:
		tween.kill()
	self.progress_rect.material.set_shader_parameter("value", progress)


func _on_frob_description_deactivated():
	self.hide()
