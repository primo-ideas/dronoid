extends Node2D

class_name Entity

static var mineral_texture = preload("res://assets/textures/mineral.png")
static var dronoid_texture = preload("res://assets/textures/dronoid.png")
static var factory_texture = preload("res://assets/textures/factory.png")
static var beacon_texture = preload("res://assets/textures/beacon.png")

static var entity_scene = preload("res://entity.tscn")

var sprite: Sprite2D:
	get: return $Sprite2D

var area: Area2D:
	get: return $Area2D

func set_texture(texture: Texture2D, size: int):
	sprite.texture = texture
	sprite.scale = (Vector2.ONE / texture.get_size()) * size

func disable_mouse_interaction():
	area.process_mode = Node.PROCESS_MODE_DISABLED

func _on_area_2d_mouse_entered() -> void:
	# sprite.modulate
	pass

static func create(kind: String, ):
	var entity_node = entity_scene.instantiate()
	if kind == "Mineral":
		entity_node.set_texture(mineral_texture, 1)
		entity_node.disable_mouse_interaction()
	elif kind == "Dronoid":
		entity_node.set_texture(dronoid_texture, 3)
	elif kind == "Factory":
		entity_node.set_texture(factory_texture, 6)
	elif kind == "Beacon":
		entity_node.set_texture(beacon_texture, 9)
		entity_node.disable_mouse_interaction()
	else:
		printerr("Unknown kind")
	return entity_node
