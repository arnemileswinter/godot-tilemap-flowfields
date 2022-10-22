extends KinematicBody2D

var _velocity = Vector2.ZERO
var _tile_map : TileMap
var _baked_flow_fields : Resource
var _target_world_position

func walk_to_world_position(world_pos : Vector2):
	_target_world_position = world_pos

func set_tilemap(t:TileMap):
	_tile_map = t

func set_baked_flow_fields(f: Resource):
	_baked_flow_fields = f

func _physics_process(delta):
	var new_velocity = Vector2.ZERO
	if _baked_flow_fields and _tile_map and _target_world_position:
		var current_map_position : Vector2 = _tile_map.world_to_map(_tile_map.to_local(global_position))
		var target_map_position : Vector2 = _tile_map.world_to_map(_tile_map.to_local(_target_world_position))
		new_velocity = _baked_flow_fields.flow_from_to(current_map_position, target_map_position)
		rotation = new_velocity.angle()
	_velocity = move_and_slide(new_velocity * 50)
