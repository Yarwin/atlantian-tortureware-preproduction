extends SubViewportContainer

@onready var viewport := $SubViewport

func _input(event: InputEvent) -> void:
	viewport.push_input(event)
