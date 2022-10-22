extends Area2D

var _tile_map : TileMap setget set_tile_map
var _flow_field : Resource setget set_flow_field

var _velocity = Vector2.ZERO

func set_tile_map(m:TileMap):
	_tile_map = m
	_align_orientation()

func set_flow_field(f: Resource):
	_flow_field = f
	_align_orientation()

func _align_orientation():
	if _flow_field and _tile_map:
		var current_map_position : Vector2 = _tile_map.world_to_map(_tile_map.to_local(global_position))
		if _flow_field.can_flow(current_map_position):
			visible = true
			var flow_field_vec = _flow_field.flow(current_map_position)
			rotation = flow_field_vec.angle()
		else:
			visible = false
