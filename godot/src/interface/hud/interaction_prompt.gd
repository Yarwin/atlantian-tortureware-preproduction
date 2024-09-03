extends Control
@onready var frob_info := $MarginContainer/HBoxContainer/FrobInfo
@onready var use_label := $MarginContainer/HBoxContainer/Label
@onready var progress_rect := $MarginContainer/TextureRect
@onready var name_display := $MarginContainer/HBoxContainer/Label


func _ready() -> void:
	self.hide()
	GameSystems.frob_prompt_updated.connect(_on_frob_prompt_updated)
	GameSystems.frob_description_deactivated.connect(_on_frob_description_deactivated)
	GameSystems.frob_progress_updated.connect(_on_frob_progress_updated)
	

func _on_frob_prompt_updated(new: String, progress: float, display: String):
	self.progress_rect.material.set_shader_parameter("value", 0.0)
	self.show()
	frob_info.text = new
	name_display.text = display
	var use_label_size = use_label.get_rect().size


func _on_frob_progress_updated(progress: float):
	self.progress_rect.material.set_shader_parameter("value", progress)


func _on_frob_description_deactivated():
	self.hide()
