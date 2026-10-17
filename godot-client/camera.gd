# By Claude Sonnet

extends Camera2D

const ZOOM_STEP := 1.1
const ZOOM_MIN := 1.0
const ZOOM_MAX := 10
const ZOOM_START := 7.

var _dragging := false

func _ready() -> void:
	zoom = Vector2(ZOOM_START, ZOOM_START)

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseButton:
		match event.button_index:
			MOUSE_BUTTON_WHEEL_UP:
				if event.pressed:
					_zoom_at(ZOOM_STEP, event.position)
			MOUSE_BUTTON_WHEEL_DOWN:
				if event.pressed:
					_zoom_at(1.0 / ZOOM_STEP, event.position)
			MOUSE_BUTTON_MIDDLE, MOUSE_BUTTON_RIGHT:
				_dragging = event.pressed
	elif event is InputEventMouseMotion and _dragging:
		global_position -= event.relative / zoom

func _zoom_at(factor: float, screen_point: Vector2) -> void:
	var old_zoom := zoom
	var new_zoom := (old_zoom * factor).clamp(
		Vector2.ONE * ZOOM_MIN, Vector2.ONE * ZOOM_MAX)
	if new_zoom == old_zoom:
		return
	var from_center := screen_point - get_viewport_rect().size * 0.5
	global_position += from_center * (Vector2.ONE / old_zoom - Vector2.ONE / new_zoom)
	zoom = new_zoom
