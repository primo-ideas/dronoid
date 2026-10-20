extends Node

enum NetState {
	AUTHENTICATION_SENDING_REQUEST,
	AUTHENTICATION_WAITING_RESPONSE,
	PLAYING
}

@onready var root = get_tree().get_first_node_in_group("root")
@onready var ui = root.ui
@onready var mineral_texture = preload("res://assets/textures/mineral.png")
@onready var dronoid_texture = preload("res://assets/textures/dronoid.png")
@onready var factory_texture = preload("res://assets/textures/factory.png")
@onready var beacon_texture = preload("res://assets/textures/beacon.png")

var uri = "ws://127.0.0.1:8080/dronoid/ws"
var state = null;
var socket = WebSocketPeer.new()
var player_nickname = null
var entities = {}

func _ready() -> void:
	set_process(false)

func send(message: Variant):
	socket.send_text(JSON.stringify(message))

func connect_to_server(nickname: String):
	if is_processing():
		return
	var err = socket.connect_to_url(uri)
	if err != OK:
		ui.info_label.text = "Can't connect to %s" % [uri]
		set_process(false)
		return
	socket.inbound_buffer_size = 10000000
	socket.max_queued_packets = 10000
	socket.outbound_buffer_size = 10000000
	self.player_nickname = nickname
	state = NetState.AUTHENTICATION_SENDING_REQUEST
	set_process(true)

	ui.title.visible = false
	ui.connect_panel.visible = false
	ui.info_label.text = "Connecting to %s" % [uri]

func next_state_packet():
	var packet = socket.get_packet()
	if !socket.was_string_packet():
		ui.info_label.text = "Unexpected data from server"
		socket.close()
		return
	var packet_text = packet.get_string_from_utf8()
	var server_message = JSON.parse_string(packet_text)
	var game_state = server_message
	var entities_in_zone: Array = game_state["visible_minerals"]

	var _self_player = game_state["player_info"]["minerals_cnt"]

	for entity in entities_in_zone:
		if entities.has(entity.id):
			var existing_entity = entities.get(entity.id) as Node2D
			existing_entity.position.x = entity.pos[0]
			existing_entity.position.y = entity.pos[1]
			# existing_entity.position.z = voxel.pos_z
		else:
			var game_sprite = Sprite2D.new()
			var size = 1
			if entity.kind == "Mineral":
				game_sprite.texture = mineral_texture
			elif entity.kind == "Dronoid":
				size = 3
				game_sprite.texture = dronoid_texture
			elif entity.kind == "Factory":
				size = 6
				game_sprite.texture = factory_texture
			elif entity.kind == "Beacon":
				size = 9
				game_sprite.texture = beacon_texture
			else:
				printerr("Unknown kind")
			var node = Node2D.new()

			node.add_child(game_sprite)

			root.world.add_child(node)
			game_sprite.scale = (Vector2.ONE / game_sprite.texture.get_size()) * size

			node.position.x = entity.pos[0]
			node.position.y = entity.pos[1]
			node.name = "%s_%d" % [entity.kind, entity.id]
			entities[entity.id] = node

func handle_net():
	if state == NetState.AUTHENTICATION_SENDING_REQUEST:
		ui.info_label.text = "Authenticating"
		socket.send_text(JSON.stringify(
			{"nickname": self.player_nickname}
		))
		state = NetState.AUTHENTICATION_WAITING_RESPONSE
	elif state == NetState.AUTHENTICATION_WAITING_RESPONSE:
		if socket.get_available_packet_count() == 0:
			return
		var packet = socket.get_packet()
		if !socket.was_string_packet():
			ui.info_label.text = "Unexpected data from server"
			socket.close()
			return
		var packet_text = packet.get_string_from_utf8()
		print(packet_text)
		var auth_response = JSON.parse_string(packet_text)
		print(auth_response)
		if auth_response["result"] == false:
			ui.info_label.text = "Server declined authentication: %s" % [auth_response["message"]]
			socket.close()
			return
		ui.info_label.text = ""
		root.camera.position = Vector2(auth_response.spawn[0], auth_response.spawn[1])
		state = NetState.PLAYING
		ui.game_panel.visible = true
	elif state == NetState.PLAYING:
		while socket.get_available_packet_count() > 0:
			next_state_packet()
						
func _process(_delta):
	socket.poll()
	
	var socket_state = socket.get_ready_state()

	if socket_state == WebSocketPeer.STATE_OPEN:
		handle_net()
	elif socket_state == WebSocketPeer.STATE_CLOSED:
		var code = socket.get_close_code()
		ui.info_label.text = "WebSocket closed with code: %d" % [code]
		set_process(false)
		ui.title.visible = false
		ui.connect_panel.visible = false
