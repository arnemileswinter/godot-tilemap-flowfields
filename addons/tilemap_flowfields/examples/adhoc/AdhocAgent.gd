extends KinematicBody2D

var _tile_map : TileMap
var _flow_field : Resource

var _velocity = Vector2.ZERO

func set_tilemap(t:TileMap):
	_tile_map = t

func set_flow_field(f: Resource):
	_flow_field = f

func _physics_process(delta):
	var new_velocity = Vector2.ZERO
	if _flow_field and _tile_map:
		var current_map_position : Vector2 = _tile_map.world_to_map(_tile_map.to_local(global_position))
		new_velocity = _flow_field.flow(current_map_position)
		rotation = new_velocity.angle()
	_velocity = move_and_slide(new_velocity * 50)
