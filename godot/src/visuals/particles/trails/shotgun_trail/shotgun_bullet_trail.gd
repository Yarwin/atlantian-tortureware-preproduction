extends GPUParticles3D

func _ready() -> void:
	top_level = true
	restart()

func _on_finished() -> void:
	queue_free()
