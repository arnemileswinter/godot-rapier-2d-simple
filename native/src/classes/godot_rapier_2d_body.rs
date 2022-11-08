use std::rc::Rc;
use std::{cell::RefCell, rc::Weak};

use gdnative::prelude::*;
use rapier2d::prelude as r2d;

use super::common::{not_in_editor, PixelsPerMeter, World2DAware, World2DState};
use super::godot_rapier_2d_collider::{GodotRapier2DColliderBall, GodotRapier2DColliderCuboid};

#[derive(Default)]
struct WorldProps {
    world_ref: Option<Weak<RefCell<World2DState>>>,
    world_ppm: Option<PixelsPerMeter>,
    handle: Option<r2d::RigidBodyHandle>,
}
macro_rules! proxy_world_awareness_to_world_props {
    ($t:ty) => {
        impl World2DAware<TRef<'_, Node2D>> for $t {
            type Handle = r2d::RigidBodyHandle;

            fn add_to_world(
                &mut self,
                base: TRef<'_, Node2D>,
                world_ref: &super::common::World2DRef,
                handle: Self::Handle,
                ppm: PixelsPerMeter,
            ) {
                self.world_props.world_ref = Some(Rc::downgrade(world_ref));
                self.world_props.world_ppm = Some(ppm);
                self.world_props.handle = Some(handle);

                base.get_children().iter().for_each(|c| {
                    let Ok(n) = c.try_to() else {return;};
                    self._unregister_child(base, n);
                });
                base.get_children().iter().for_each(|c| {
                    let Ok(n) = c.try_to() else {return;};
                    self._register_child(base, n);
                })
            }

            fn remove_from_world(&mut self, base: TRef<'_, Node2D>) {
                base.get_children().iter().for_each(|c| {
                    let Ok(n) = c.try_to() else {return;};
                    self._unregister_child(base, n);
                });

                if let Some(world) = self.world_props.world_ref.as_ref().and_then(Weak::upgrade) {
                    if let Some(handle) = self.world_props.handle {
                        world.borrow_mut().remove_rigid_body(handle)
                    }
                };

                self.world_props.world_ref = None;
                self.world_props.handle = None;
                self.world_props.world_ppm = None;
            }
        }
    };
}

// bridges common props towards collider builder.
// stores a raw pointer box to reference of `base` on the heap, which must be dropped eventually.
macro_rules! complete_body {
    ($self:ident,$base:ident,$ppm:ident,$builder:expr) => {{
        use rapier2d::prelude::*;
        $builder
            .translation(vector![
                $base.position().x / $ppm.0,
                $base.position().y / $ppm.0
            ])
            .rotation($base.rotation() as f32)
            .user_data(&$base.claim() as *const Ref<Node2D> as u128)
            .build()
    }};
}

#[derive(NativeClass)]
#[inherit(Node2D)]
pub struct GodotRapier2DFixedBody {
    world_props: WorldProps,
}
proxy_world_awareness_to_world_props!(GodotRapier2DFixedBody);
#[derive(NativeClass)]
#[inherit(Node2D)]
pub struct GodotRapier2DDynamicBody {
    world_props: WorldProps,
}
proxy_world_awareness_to_world_props!(GodotRapier2DDynamicBody);
#[derive(NativeClass)]
#[inherit(Node2D)]
pub struct GodotRapier2DKinematicPositionBasedBody {
    world_props: WorldProps,
}
proxy_world_awareness_to_world_props!(GodotRapier2DKinematicPositionBasedBody);
#[derive(NativeClass)]
#[inherit(Node2D)]
pub struct GodotRapier2DKinematicVelocityBasedBody {
    world_props: WorldProps,
}
proxy_world_awareness_to_world_props!(GodotRapier2DKinematicVelocityBasedBody);

impl GodotRapier2DFixedBody {
    pub fn build_rigid_body(&self, base: TRef<'_, Node2D>, ppm: PixelsPerMeter) -> r2d::RigidBody {
        complete_body!(self, base, ppm, r2d::RigidBodyBuilder::fixed())
    }
}
impl GodotRapier2DDynamicBody {
    pub fn build_rigid_body(&self, base: TRef<'_, Node2D>, ppm: PixelsPerMeter) -> r2d::RigidBody {
        complete_body!(self, base, ppm, r2d::RigidBodyBuilder::dynamic())
    }
}
impl GodotRapier2DKinematicPositionBasedBody {
    pub fn build_rigid_body(&self, base: TRef<'_, Node2D>, ppm: PixelsPerMeter) -> r2d::RigidBody {
        complete_body!(
            self,
            base,
            ppm,
            r2d::RigidBodyBuilder::kinematic_position_based()
        )
    }
}
impl GodotRapier2DKinematicVelocityBasedBody {
    pub fn build_rigid_body(&self, base: TRef<'_, Node2D>, ppm: PixelsPerMeter) -> r2d::RigidBody {
        complete_body!(
            self,
            base,
            ppm,
            r2d::RigidBodyBuilder::kinematic_velocity_based()
        )
    }
}

macro_rules! gen_godot_impl {
    ($t:ty) => {
        #[methods]
        impl $t {
            fn new(_base: &Node2D) -> Self {
                Self {
                    world_props: WorldProps::default(),
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
            fn add_child(
                &mut self,
                #[base] base: TRef<'_, Node2D>,
                node: Ref<Node>) {
                base.add_child(node, false);
                self._register_child(base, node)
            }

            #[method]
            fn _physics_process(&mut self, #[base] base: &Node2D, _delta: f32) {
                not_in_editor!();
                let Some(world_ref) = &self.world_props.world_ref else {return};
                let Some(handle) = self.world_props.handle else {return};
                let Some(world_ref) = world_ref.upgrade() else {
                    self.world_props.handle = None;
                    return
                };
                let Some(ppm) = self.world_props.world_ppm else {return};

                let world = (*world_ref).borrow();
                let Some(body) = world.rigid_body_set.get(handle) else {return};
                let iso = body.position();

                // copy rapier isometry onto godot object.
                base.set_global_position(Vector2 {
                    x: iso.translation.x * ppm.0,
                    y: iso.translation.y * ppm.0,
                });
                base.set_global_rotation(iso.rotation.angle() as f64)
            }

            #[method]
            fn _register_child(&mut self, #[base] base: TRef<'_, Node2D>, node: Ref<Node>) {
                let Some (parent_handle) = self.world_props.handle else {return};
                let Some (ppm) = self.world_props.world_ppm else {return};
                let Some (world_ref) = self.world_props.world_ref.as_ref().and_then(Weak::upgrade) else { return };
                let Some(n) : Option<TRef<'_,Node2D>> = unsafe { node.assume_safe() }.cast() else {return};
                    macro_rules! register_collider {
                        ($i:ident) => {{
                            $i.map_mut(move |n, b| {
                                let c = n.build_collider(b, ppm);
                                let handle = world_ref
                                    .borrow_mut()
                                    .insert_collider_with_parent(c, parent_handle);
                                n.add_to_world(base, &world_ref, handle, ppm)
                            })
                            .unwrap();
                            n.connect(
                                "tree_exiting",
                                base,
                                "_unregister_child",
                                (VariantArray::from_iter(vec![node])).into_shared(),
                                Object::CONNECT_ONESHOT,
                            )
                            .unwrap();
                        }};
                    }

                    if let Some(i) = n.cast_instance::<GodotRapier2DColliderBall>() {
                        register_collider!(i)
                    } else if let Some(i) = n.cast_instance::<GodotRapier2DColliderCuboid>() {
                        register_collider!(i)
                    }
            }

            #[method]
            fn _unregister_child(&mut self, #[base] _base: TRef<'_, Node2D>, node: Ref<Node>) {
                let n = unsafe { node.assume_safe() };
                if let Some(n2d) = n.cast() {
                    let _: TRef<'_, Node2D> = n2d; // help type inference

                    macro_rules! unregister_collider {
                        ($i:ident) => {{
                            $i.map_mut(|n, b| n.remove_from_world(b)).unwrap()
                        }}
                    }

                    if let Some(i) = n2d.cast_instance::<GodotRapier2DColliderBall>() {
                        unregister_collider!(i)
                    } else if let Some(i) = n2d.cast_instance::<GodotRapier2DColliderCuboid>() {
                        unregister_collider!(i)
                    }
                }
            }
        }
    }
}

gen_godot_impl!(GodotRapier2DFixedBody);
gen_godot_impl!(GodotRapier2DDynamicBody);
gen_godot_impl!(GodotRapier2DKinematicPositionBasedBody);
gen_godot_impl!(GodotRapier2DKinematicVelocityBasedBody);
