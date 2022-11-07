use gdnative::prelude::*;

mod classes;

fn init(handle: InitHandle) {
    handle.add_tool_class_as::<classes::godot_rapier_2d_world::GodotRapier2DWorld>(
        "Rapier2DWorld".into(),
    );
    handle.add_tool_class_as::<classes::godot_rapier_2d_body::GodotRapier2DFixedBody>(
        "Rapier2DFixedBody".into(),
    );
    handle.add_tool_class_as::<classes::godot_rapier_2d_body::GodotRapier2DDynamicBody>(
        "Rapier2DDynamicBody".into(),
    );
    handle.add_tool_class_as::<classes::godot_rapier_2d_body::GodotRapier2DKinematicPositionBasedBody>(
        "Rapier2DKinematicPositionBasedBody".into(),
    );
    handle.add_tool_class_as::<classes::godot_rapier_2d_body::GodotRapier2DKinematicVelocityBasedBody>(
        "Rapier2DKinematicVelocityBasedBody".into(),
    );
    handle.add_tool_class_as::<classes::godot_rapier_2d_collider::GodotRapier2DColliderBall>(
        "Rapier2DBallCollider".into(),
    );
    handle.add_tool_class_as::<classes::godot_rapier_2d_collider::GodotRapier2DColliderCuboid>(
        "Rapier2DCuboidCollider".into(),
    );
}

godot_init!(init);
