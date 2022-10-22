extends Node2D

export (bool) var generate_debug_vectors : bool = false 

onready var _tile_map = $TileMap
onready var _flow_field_gen = $FlowFieldGenerator

var _debug_vecs := []
var _agents := []

var _agent_pre := preload("AdhocAgent.tscn")

func _unhandled_input(event):
	if event is InputEventKey and event.is_pressed() and event.scancode == KEY_SPACE:
		var agent = _agent_pre.instance()
		_agents.append(agent)
		agent.global_position = get_global_mouse_position()
		agent.set_tilemap(_tile_map)
		_tile_map.add_child(agent)
	if event is InputEventMouseButton and event.button_index == BUTTON_LEFT:
		var mouse_tile_pos = _tile_map.world_to_map(_tile_map.to_local(get_global_mouse_position()))
		var ff : Resource = _flow_field_gen.calculate_flow_field(mouse_tile_pos)
		for v in _debug_vecs:
			v.set_flow_field(ff)
		for agent in _agents:
			agent.set_flow_field(ff)

func _ready():
	if generate_debug_vectors:
		for x in range(0,_tile_map.get_used_rect().size.x):
			for y in range(0,_tile_map.get_used_rect().size.y):
				var v = preload("../DebugVector.tscn").instance()
				_tile_map.add_child(v)
				v.set_tile_map(_tile_map)
				v.global_position = _tile_map.to_global(_tile_map.map_to_world(Vector2(x + .5,y + .5)))
				_debug_vecs.append(v)
