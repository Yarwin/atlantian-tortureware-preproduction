extends Decal


func _ready() -> void:
	var tween = create_tween()
	tween.tween_property(self, "modulate:a", 0, 2.0).set_delay(1.0)
	tween.tween_callback(self.queue_free)
