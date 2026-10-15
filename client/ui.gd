extends Control

@onready var root = get_tree().get_first_node_in_group("root")
@onready var net = root.net
@onready var info_label = $InfoLabel
@onready var nickname_line_edit = $CenterContainer/ConnectPanel/MarginContainer/HBoxContainer/NicknameLineEdit

func _on_connect_button_pressed() -> void:
	net.connect_to_server(nickname_line_edit.text)
