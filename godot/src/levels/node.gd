extends Node

class_name Dupa

func _ready() -> void:
	print(
		ClassDB.class_exists("Dupa")
	)
	print(ClassDB.get_class_list())
