tool
extends Node2D

onready var _tile_map = $TileMap
onready var _flow_field_gen = $FlowFieldGenerator
var _baked_flow_fields : Resource

export(bool) var bake := false setget do_bake
export(String) var bake_to_file_path := "res://addons/tilemap_flowfields/examples/baked/baked_flow_field.res"

var _agents := []
var _agent_pre := preload("BakedAgent.tscn")

func do_bake(new):
	bake = new
	if bake:
		var bake = _flow_field_gen.bake_flowfields()
		ResourceSaver.save(bake_to_file_path, bake)
	bake = false

func _ready():
	if Engine.is_editor_hint():
		return
	_baked_flow_fields = ResourceLoader.load(bake_to_file_path)

func _unhandled_input(event):
	if Engine.is_editor_hint():
		return
	if event is InputEventKey and event.is_pressed() and event.scancode == KEY_SPACE:
		var agent = _agent_pre.instance()
		_agents.append(agent)
		agent.global_position = get_global_mouse_position()
		agent.set_tilemap(_tile_map)
		agent.set_baked_flow_fields(_baked_flow_fields)
		_tile_map.add_child(agent)
	if event is InputEventMouseButton and event.button_index == BUTTON_LEFT:
		var mouse_tile_pos = get_global_mouse_position()
		for agent in _agents:
			agent.walk_to_world_position(mouse_tile_pos)
