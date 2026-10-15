extends Node

enum NetState {
	AUTHENTICATION_SENDING_REQUEST,
	AUTHENTICATION_WAITING_RESPONSE,
	PLAYING
}

@onready var root = get_tree().get_first_node_in_group("root")
@onready var ui = root.ui

var uri = "ws://127.0.0.1:8080"
# var messages_to_send = []
var state = null;
var socket = WebSocketPeer.new()
var player_nickname = null

#var tile_scene = preload('res://tile.tscn')

# var voxels_in_game = {}

func _ready() -> void:
	set_process(false)

# func push_action(action_type, action: Variant):
# 	var next_action_message;
	
# 	if action_type == 0:
# 		next_action_message = {"action_type": 0}
# 	else:
# 		assert(false)

# 	messages_to_send.push_back(next_action_message)
# 	messages_to_send.push_back(action)

func connect_to_server(nickname: String):
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
	ui.center_container.visible = false
	ui.title.visible = false
	ui.info_label.text = "Connecting to %s" % [uri]

func handle_net():
	# for message in messages_to_send:
	# 	socket.send_text(JSON.stringify(message))
	# messages_to_send.clear()
	if state == NetState.AUTHENTICATION_SENDING_REQUEST:
		ui.info_label.text = "Authenticating"
		socket.send_text(JSON.stringify({
			"AuthenticationRequest": {"nickname": self.player_nickname}
		}))
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
		if auth_response["result"] == false:
			ui.info_label.text = "Server declined authentication: %s" % [auth_response["message"]]
			socket.close()
			return
		ui.info_label.text = ""
		# ui.center_container.visible = false
		# root.player.visible = true
		# root.player.set_process(true)
		# root.player.set_visible(true)
		state = NetState.PLAYING
	elif state == NetState.PLAYING:
		while socket.get_available_packet_count() > 0:
			var packet = socket.get_packet()
			if !socket.was_string_packet():
				ui.info_label.text = "Unexpected data from server"
				socket.close()
				return
			var packet_text = packet.get_string_from_utf8()
			var server_message = JSON.parse_string(packet_text)
			# var voxels = server_message["voxels"] as Array
			# #var players = data["players"] as Array
			# var self_player = server_message["player"]
			# root.player.position.x = self_player.pos_x
			# root.player.position.y = self_player.pos_y
			# root.player.position.z = self_player.pos_z
			# var direction = Vector3()
			# direction.x = self_player.dir_x
			# direction.y = self_player.dir_y
			# direction.z = self_player.dir_z
			# # root.player.set_direction(direction)
			
			# ui.info_label.text = "(%d, %d, %d) (%d, %d, %d)" % [root.player.position.x, root.player.position.y,
			# root.player.position.z, direction.x, direction.y, direction.z]
			# for voxel in voxels:
			# 	if voxels_in_game.has(voxel.id):
			# 		var existing_voxel = voxels_in_game.get(voxel.id) as Node3D
			# 		existing_voxel.position.x = voxel.pos_x
			# 		existing_voxel.position.y = voxel.pos_y
			# 		existing_voxel.position.z = voxel.pos_z
			# 	else:
			# 		var tile = tile_scene.instantiate()
			# 		root.add_child(tile)
			# 		tile.position.x = voxel.pos_x
			# 		tile.position.y = voxel.pos_y
			# 		tile.position.z = voxel.pos_z
			# 		voxels_in_game[voxel.id] = tile
						
func _process(_delta):
	socket.poll()
	
	var socket_state = socket.get_ready_state()

	if socket_state == WebSocketPeer.STATE_OPEN:
		handle_net()
	elif socket_state == WebSocketPeer.STATE_CLOSED:
		# root.player.set_process(false)
		var code = socket.get_close_code()
		ui.info_label.text = "WebSocket closed with code: %d" % [code]
		set_process(false)
