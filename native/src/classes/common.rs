use rapier2d::prelude::{self as r2d};
use std::{cell::RefCell, rc::Rc};

pub type World2DRef = Rc<RefCell<World2DState>>;

pub trait World2DAware<T> {
    type Handle;
    fn add_to_world(
        &mut self,
        base: T,
        world_ref: &World2DRef,
        handle: Self::Handle,
        ppm: PixelsPerMeter,
    );
    fn remove_from_world(&mut self, base: T);
}

#[derive(Default)]
pub struct World2DState {
    pub physics_pipeline: r2d::PhysicsPipeline,
    pub gravity: r2d::Vector<f32>,
    pub integration_parameters: r2d::IntegrationParameters,
    pub island_manager: r2d::IslandManager,
    pub broad_phase: r2d::BroadPhase,
    pub narrow_phase: r2d::NarrowPhase,
    pub rigid_body_set: r2d::RigidBodySet,
    pub collider_set: r2d::ColliderSet,
    pub impulse_joints: r2d::ImpulseJointSet,
    pub multibody_joints: r2d::MultibodyJointSet,
    pub ccd_solver: r2d::CCDSolver,
    pub hooks: (),
    pub events: RawRefCollisionEventHandler,
}

#[derive(Default)]
pub struct RawRefCollisionEventHandler {}
impl r2d::EventHandler for RawRefCollisionEventHandler {
    fn handle_collision_event(
        &self,
        _bodies: &r2d::RigidBodySet,
        colliders: &r2d::ColliderSet,
        event: r2d::CollisionEvent,
        _contact_pair: Option<&r2d::ContactPair>,
    ) {
        println!("event!");
        use gdnative::prelude::*;

        let collider_1 = &colliders[event.collider1()];
        let collider_2 = &colliders[event.collider2()];

        let collider_1_node_ref_ptr = collider_1.user_data as *const Ref<Node2D>;
        let collider_2_node_ref_ptr = collider_2.user_data as *const Ref<Node2D>;

        let collider_1_node_ref = if collider_1_node_ref_ptr.is_null() {
            panic!("collider 1 has no node2d user data assigned!")
        } else {
            unsafe { *collider_1_node_ref_ptr }
        };
        let collider_2_node_ref = if collider_2_node_ref_ptr.is_null() {
            panic!("collider 2 has no node2d user data assigned!")
        } else {
            unsafe { *collider_2_node_ref_ptr }
        };

        let Some(collider_1_node) = (unsafe { collider_1_node_ref.assume_safe_if_sane() } ) else {return};
        let Some(collider_2_node) = (unsafe { collider_2_node_ref.assume_safe_if_sane() } ) else {return};

        if collider_1.is_sensor() {
            if event.started() {
                collider_1_node
                    .emit_signal("collider_entered", &[collider_2_node_ref.to_variant()]);
            } else {
                collider_1_node.emit_signal("collider_exited", &[collider_2_node_ref.to_variant()]);
            }
        } else if collider_2.is_sensor() {
            if event.started() {
                collider_2_node
                    .emit_signal("collider_entered", &[collider_1_node_ref.to_variant()]);
            } else {
                collider_2_node.emit_signal("collider_exited", &[collider_1_node_ref.to_variant()]);
            }
        } else if event.started() {
            collider_1_node.emit_signal("collision_started", &[collider_2_node_ref.to_variant()]);
            collider_2_node.emit_signal("collision_started", &[collider_1_node_ref.to_variant()]);
        } else {
            collider_1_node.emit_signal("collision_ended", &[collider_2_node_ref.to_variant()]);
            collider_2_node.emit_signal("collision_ended", &[collider_1_node_ref.to_variant()]);
        }
    }

    fn handle_contact_force_event(
        &self,
        _dt: r2d::Real,
        _bodies: &r2d::RigidBodySet,
        _colliders: &r2d::ColliderSet,
        _contact_pair: &r2d::ContactPair,
        _total_force_magnitude: r2d::Real,
    ) {
        // intentionally left empty.
    }
}

impl World2DState {
    pub fn tick(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            &self.hooks,
            &self.events,
        )
    }

    pub fn insert_collider_with_parent(
        &mut self,
        coll: r2d::Collider,
        parent_handle: r2d::RigidBodyHandle,
    ) -> r2d::ColliderHandle {
        self.collider_set
            .insert_with_parent(coll, parent_handle, &mut self.rigid_body_set)
    }

    pub fn remove_rigid_body(&mut self, handle: r2d::RigidBodyHandle) {
        self.rigid_body_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            false,
        );
    }

    pub fn remove_collider(&mut self, handle: r2d::ColliderHandle) {
        self.collider_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.rigid_body_set,
            true,
        );
    }
}

#[derive(Clone, Copy)]
pub struct PixelsPerMeter(pub f32);

macro_rules! not_in_editor {
    () => {
        use gdnative::api::Engine;
        if Engine::is_editor_hint(Engine::godot_singleton()) {
            return;
        }
    };
}
pub(crate) use not_in_editor;
