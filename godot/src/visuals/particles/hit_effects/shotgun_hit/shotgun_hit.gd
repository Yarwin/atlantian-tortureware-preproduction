extends Node3D


func _on_gpu_particles_3d_finished() -> void:
	queue_free()
