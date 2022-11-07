use rapier2d::prelude as r2d;
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
    pub events: (),
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
