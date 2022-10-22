tool
extends EditorPlugin


func _enter_tree():
	add_custom_type("FlowFieldGenerator", "Node", preload("res://addons/tilemap_flowfields/classes/flow_field_generator.gdns"), null)
	add_custom_type("FlowFieldTileCost", "Node", preload("res://addons/tilemap_flowfields/classes/flow_field_tile_cost.gdns"), null)
	add_custom_type("FlowField", "Resource", preload("res://addons/tilemap_flowfields/classes/flow_field.gdns"), null)
	add_custom_type("BakedFlowFields", "Resource", preload("res://addons/tilemap_flowfields/classes/baked_flow_fields.gdns"), null)


func _exit_tree():
	remove_custom_type("FlowFieldGenerator")
	remove_custom_type("FlowFieldTileCost")
	remove_custom_type("FlowField")
	remove_custom_type("BakedFlowFields")
