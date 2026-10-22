extends Control

@onready var root = get_tree().get_first_node_in_group("root")
@onready var net = root.net
@onready var connect_panel = $CenterContainer/ConnectPanel
@onready var nickname_line_edit = $CenterContainer/ConnectPanel/HBoxContainer/NicknameLineEdit
@onready var info_label = $InfoLabel
@onready var debug_label = $DebugLabel
@onready var title = $Title
@onready var game_panel = $PanelContainer

@onready var factory_texture = root.net.factory_texture

var placing_building = false
var building_node: Node2D = null

func _process(_delta):
	debug_label.text = "(%d,%d)" % [root.camera.position.x, root.camera.position.y]
	if placing_building:
		building_node.position = root.world.get_global_mouse_position()
		if Input.is_action_just_pressed("place_building"):
			net.send({"placefactory": [root.world.get_global_mouse_position().x as int, root.world.get_global_mouse_position().y as int]})

func _on_connect_button_pressed() -> void:
	net.connect_to_server(nickname_line_edit.text)

func _on_spawn_factory_button_pressed() -> void:
	if !placing_building:
		enter_placing_mode()
	else:
		leave_placing_mode()
		
func enter_placing_mode():
	building_node = Sprite2D.new()
	building_node.texture = factory_texture
	building_node.scale = Vector2.ONE / factory_texture.get_size() * 6
	root.world.add_child(building_node)
	placing_building = true

func leave_placing_mode():
	root.world.remove_child(building_node)
	building_node.queue_free()
	placing_building = false
