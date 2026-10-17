extends Node

var net: Node:
	get: return $Net

var ui: Control:
	get: return $UI/UI
	
var world: Node2D:
	get: return $World

var camera: Camera2D:
	get: return $World/Camera
