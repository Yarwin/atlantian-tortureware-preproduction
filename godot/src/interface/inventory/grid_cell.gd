extends Control

@onready var color_rect := $Panel/ColorRect

var tween
var is_highlighting: bool

func highlight():
	if tween:
		tween.kill()
	tween = create_tween()
	self.color_rect.color = Color(Color.WHITE, 0.0)
	tween.tween_property(self.color_rect, "color", Color(Color.WHITE, 0.6), 0.52)
	self.color_rect.visible = true


func highlight_red():
	if tween:
		tween.kill()
	tween = create_tween()
	self.color_rect.color = Color(Color.RED, 0.0)
	self.color_rect.visible = true
	tween.tween_property(self.color_rect, "color", Color(Color.RED, 0.6), 0.52)

func unhighlight():
	if tween:
		tween.kill()
	tween = create_tween()
	tween.tween_property(self.color_rect, "color", Color(Color.WHITE, 0.0), 0.52)
	tween.tween_callback(self.color_rect.set_visible.bind(false))
