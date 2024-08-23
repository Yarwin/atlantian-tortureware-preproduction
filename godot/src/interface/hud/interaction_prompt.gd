extends Control
@onready var frob_info := $HBoxContainer/MarginContainer/VBoxContainer/VBoxContainer/HBoxContainer/FrobInfo
@onready var use_label := $HBoxContainer/MarginContainer/VBoxContainer/VBoxContainer/HBoxContainer/UseLabel
@onready var progress_rect := $HBoxContainer/MarginContainer/VBoxContainer/VBoxContainer/HBoxContainer/UseLabel/TextureRect
@onready var name_display := $HBoxContainer/MarginContainer/VBoxContainer/VBoxContainer/Label


func _ready() -> void:
	GameSystems.frob_prompt_updated.connect(_on_frob_prompt_updated)
	GameSystems.frob_description_deactivated.connect(_on_frob_description_deactivated)
	GameSystems.frob_progress_updated.connect(_on_frob_progress_updated)
	

func _on_frob_prompt_updated(new: String, progress: float, display: String):
	self.show()
	frob_info.text = new
	name_display.text = display
	var use_label_size = use_label.get_rect().size
	progress_rect.offset_top = -use_label_size.y / 3
	progress_rect.offset_bottom = use_label_size.y / 3
	progress_rect.anchor_right = progress

func _on_frob_progress_updated(progress: float):
	progress_rect.anchor_right = progress


func _on_frob_description_deactivated():
	self.hide()
