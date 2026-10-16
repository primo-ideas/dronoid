extends SubViewportContainer

@onready var root = get_tree().get_first_node_in_group("root")
@onready var net = root.net
@onready var connect_panel = $Viewport/ConnectPanel
@onready var info_label = $Viewport/InfoLabel
@onready var debug_label = $Viewport/DebugLabel
@onready var title = $Viewport/Title
@onready var nickname_line_edit = $Viewport/ConnectPanel/Panel/MarginContainer/HBoxContainer/NicknameLineEdit

func _on_connect_button_pressed() -> void:
	net.connect_to_server(nickname_line_edit.text)

func _process(_delta: float) -> void:
	debug_label.text = "(%d,%d)" % [root.camera.position.x, root.camera.position.y]
