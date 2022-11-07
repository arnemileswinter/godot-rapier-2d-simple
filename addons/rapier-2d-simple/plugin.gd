tool
extends EditorPlugin


func _enter_tree():
	add_custom_type("Rapier2DWorld", "Node2D", preload("res://addons/rapier-2d-simple/classes/rapier_2d_world.gdns"), null)

	add_custom_type("Rapier2DFixedBody", "Node2D", preload("res://addons/rapier-2d-simple/classes/rapier_2d_fixed_body.gdns"), null)
	add_custom_type("Rapier2DDynamicBody", "Node2D", preload("res://addons/rapier-2d-simple/classes/rapier_2d_dynamic_body.gdns"), null)
	add_custom_type("Rapier2DKinematicPositionBasedBody", "Node2D", preload("res://addons/rapier-2d-simple/classes/rapier_2d_kinematic_position_based_body.gdns"), null)
	add_custom_type("Rapier2DKinematicVelocityBasedBody", "Node2D", preload("res://addons/rapier-2d-simple/classes/rapier_2d_kinematic_velocity_based_body.gdns"), null)

	add_custom_type("Rapier2DBallCollider", "Node2D", preload("res://addons/rapier-2d-simple/classes/rapier_2d_ball_collider.gdns"), null)
	add_custom_type("Rapier2DCuboidCollider", "Node2D", preload("res://addons/rapier-2d-simple/classes/rapier_2d_cuboid_collider.gdns"), null)
	


func _exit_tree():
	remove_custom_type("Rapier2DWorld")
	remove_custom_type("Rapier2DBody")

	remove_custom_type("Rapier2DFixedBody")
	remove_custom_type("Rapier2DDynamicBody")
	remove_custom_type("Rapier2DKinematicPositionBasedBody")
	remove_custom_type("Rapier2DKinematicVelocityBasedBody")

	remove_custom_type("Rapier2DBallCollider")
	remove_custom_type("Rapier2DCuboidCollider")
