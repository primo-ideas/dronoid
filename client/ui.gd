extends SubViewport

@onready var root = get_tree().get_first_node_in_group("root")
@onready var net = root.net
@onready var connect_panel = $ConnectPanel
@onready var info_label = $InfoLabel
@onready var debug_label = $DebugLabel
@onready var title = $Title
@onready var nickname_line_edit = $ConnectPanel/Panel/MarginContainer/HBoxContainer/NicknameLineEdit

func _on_connect_button_pressed() -> void:
	net.connect_to_server(nickname_line_edit.text)

func _process(_delta: float) -> void:
	debug_label.text = "(%d,%d)" % [root.camera.position.x, root.camera.position.y]
