use std::cell::RefCell;
use std::rc::Rc;

use gdnative::prelude::*;
use rapier2d::prelude as r2d;

use super::godot_rapier_body::GodotRapierBody;

#[derive(Default)]
pub struct WorldState {
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

impl WorldState {
    fn tick(&mut self) {
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
}

#[derive(NativeClass)]
#[inherit(Node2D)]
struct GodotRapierWorld {
    rapier_world: Rc<RefCell<WorldState>>,
}

impl GodotRapierWorld {}

#[methods]
impl GodotRapierWorld {
    fn new(_base: &Node2D) -> Self {
        Self {
            rapier_world: Rc::new(RefCell::new(WorldState::default())),
        }
    }

    #[method]
    fn _ready(&mut self, #[base] base: TRef<'_, Node2D>) {
        base.get_children().iter().for_each(|c| {
            let Ok(n) = c.try_to() else {return;};
            self._register_child(base, n);
        })
    }

    #[method]
    fn _physics_process(&mut self, #[base] _base: TRef<'_, Node2D>) {
        self.rapier_world.borrow_mut().tick();
    }

    #[method]
    fn add_child(
        &mut self,
        #[base] base: TRef<'_, Node2D>,
        node: Ref<Node>,
        legible_unique_name: bool,
    ) {
        base.add_child(node, legible_unique_name);
        self._register_child(base, node)
    }

    #[method]
    fn remove_child(&mut self, #[base] base: TRef<'_, Node2D>, node: Ref<Node>) {
        self._unregister_child(base, node);
        base.remove_child(node);
    }

    #[method]
    fn _register_child(&mut self, #[base] base: TRef<'_, Node2D>, node: Ref<Node>) {
        let n = unsafe { node.assume_safe() };
        if let Some(i) = n
            .cast()
            .and_then(move |o: TRef<'_, Node2D>| o.cast_instance::<GodotRapierBody>())
        {
            i.map_mut(move |n, b| {
                let handle = self
                    .rapier_world
                    .borrow_mut()
                    .rigid_body_set
                    .insert(n.build_rigid_body(b));
                n.world_ref = Some(Rc::downgrade(&self.rapier_world));
                n.handle = Some(handle);
            })
            .unwrap();
            n.connect(
                "tree_exiting",
                base,
                "_unregister_child",
                (VariantArray::from_iter(vec![node])).into_shared(),
                Object::CONNECT_ONESHOT,
            )
            .unwrap()
        }
    }

    #[method]
    fn _unregister_child(&mut self, #[base] _base: TRef<'_, Node2D>, node: Ref<Node>) {
        let n = unsafe { node.assume_safe() };
        if let Some(i) = n
            .cast()
            .and_then(move |o: TRef<'_, Node2D>| o.cast_instance::<GodotRapierBody>())
        {
            i.map_mut(|n, _| {
                n.handle = None;
                n.world_ref = None;
            })
            .unwrap()
        }
    }
}
