extends Node

enum NetState {
	AUTHENTICATION_SENDING_REQUEST,
	AUTHENTICATION_WAITING_RESPONSE,
	PLAYING
}

@onready var root = get_tree().get_first_node_in_group("root")
@onready var ui = root.ui

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

func next_play_packet():
	var packet = socket.get_packet()
	if !socket.was_string_packet():
		ui.info_label.text = "Unexpected data from server"
		socket.close()
		return
	var packet_text = packet.get_string_from_utf8()
	var server_message = JSON.parse_string(packet_text)
	if server_message.has("response"):
		if server_message.response.has("placefactory"):
			ui.leave_placing_mode()
			if !server_message.response.placefactory.result:
				ui.info_label.text = server_message.response.placefactory.message
	elif server_message.has("state"):
		var game_state = server_message
		var entities_in_zone: Array = game_state.state.visible_minerals

		var _self_player = game_state.state.player_info.minerals_cnt

		for entity in entities_in_zone:
			if entities.has(entity.id):
				var existing_entity = entities.get(entity.id) as Node2D
				existing_entity.position.x = entity.pos[0]
				existing_entity.position.y = entity.pos[1]
			else:
				var entity_node = Entity.create(entity.kind)

				root.world.add_child(entity_node)

				entity_node.position.x = entity.pos[0]
				entity_node.position.y = entity.pos[1]
				entity_node.name = "%s_%d" % [entity.kind, entity.id]
				entities[entity.id] = entity_node

func handle_transport():
	if state == NetState.AUTHENTICATION_SENDING_REQUEST:
		ui.info_label.text = "Authenticating"
		socket.send_text(JSON.stringify(
			{"authenticationrequest": {"nickname": self.player_nickname}}
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
		var auth_response = JSON.parse_string(packet_text)
		if auth_response.response.authentication.result == false:
			ui.info_label.text = "Server declined authentication: %s" % [auth_response.response.authentication.message]
			socket.close()
			return
		ui.info_label.text = ""
		root.camera.position = Vector2(auth_response.response.authentication.spawn[0], auth_response.response.authentication.spawn[1])
		state = NetState.PLAYING
		ui.game_panel.visible = true
		ui.factory_panel.visible = true
	elif state == NetState.PLAYING:
		while socket.get_available_packet_count() > 0:
			next_play_packet()
						
func _process(_delta):
	socket.poll()
	
	var socket_state = socket.get_ready_state()

	if socket_state == WebSocketPeer.STATE_OPEN:
		handle_transport()
	elif socket_state == WebSocketPeer.STATE_CLOSED:
		var code = socket.get_close_code()
		ui.info_label.text = "WebSocket closed with code: %d" % [code]
		set_process(false)
		ui.title.visible = false
		ui.connect_panel.visible = false
