use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use gdnative::prelude::*;
use rapier2d::prelude::{self as r2d, ColliderBuilder};

use super::common::{PixelsPerMeter, World2DAware, World2DState};

macro_rules! shape_color {
    () => {
        Color::from_html("88FF0000").unwrap()
    };
}

macro_rules! should_draw {
    ($base : ident) => {{
        use gdnative::api::Engine;
        Engine::godot_singleton().is_editor_hint()
            || $base
                .get_tree()
                .and_then(|t| unsafe { t.assume_safe_if_sane() })
                .map(|a| a.is_debugging_collisions_hint())
                .unwrap_or(true)
    }};
}

#[derive(Default)]
struct WorldProps {
    world_ref: Option<Weak<RefCell<World2DState>>>,
    world_ppm: Option<PixelsPerMeter>,
    handle: Option<r2d::ColliderHandle>,
}

// receive collider handle or abort current routine.
macro_rules! handle_or_return {
    ($s:ident,$h:ident) => {
        let Some($h) = $s.world_props.handle else {return};
    };
}

// receive mutable world ref or abort current routine.
macro_rules! mut_world_or_return {
    ($s:ident,$i:ident) => {
        let Some(world) = $s.world_props.world_ref.as_ref().and_then(Weak::upgrade) else {return};
        let mut $i = world.borrow_mut();
    };
}

// Convenience macros for collider structs to proxy world awareness to ColliderData.
macro_rules! proxy_world_awareness_to_world_props {
    ($t:ty) => {
        impl World2DAware<TRef<'_, Node2D>> for $t {
            type Handle = r2d::ColliderHandle;

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
                base.update()
            }

            fn remove_from_world(&mut self, base: TRef<'_, Node2D>) {
                self.world_props.world_ref = None;
                self.world_props.handle = None;
                self.world_props.world_ppm = None;
                base.update();
            }
        }
    };
}

// common properties for colliders.
struct ColliderProps {
    friction: f32,
    density: f32,
    restitution: f32,
    is_sensor: bool,
}
impl Default for ColliderProps {
    fn default() -> Self {
        Self {
            friction: ColliderBuilder::default_friction(),
            density: ColliderBuilder::default_density(),
            restitution: 0.0,
            is_sensor: false,
        }
    }
}
// bridges godot editor property setting to colliderprops member.
macro_rules! proxy_properties_to_collider_props {
    ($builder: ident) => {
        $builder
            .property("friction")
            .with_default(ColliderBuilder::default_friction())
            .with_getter(|s, _| s.collider_props.friction)
            .with_setter(|s, _, new_val| {
                s.collider_props.friction = new_val;

                handle_or_return!(s, handle);
                mut_world_or_return!(s, world);
                world.collider_set[handle].set_friction(new_val);
            })
            .done();
        $builder
            .property("density")
            .with_default(ColliderBuilder::default_density())
            .with_getter(|s, _| s.collider_props.density)
            .with_setter(|s, _, new_val| {
                s.collider_props.density = new_val;

                handle_or_return!(s, handle);
                mut_world_or_return!(s, world);
                world.collider_set[handle].set_density(new_val);
            })
            .done();
        $builder
            .property("restitution")
            .with_default(0.)
            .with_getter(|s, _| s.collider_props.restitution)
            .with_setter(|s, _, new_val| {
                s.collider_props.restitution = new_val;
                handle_or_return!(s, handle);
                mut_world_or_return!(s, world);
                world.collider_set[handle].set_restitution(new_val);
            })
            .done();
        $builder
            .property("is_sensor")
            .with_default(false)
            .with_getter(|s, _| s.collider_props.is_sensor)
            .with_setter(|s, _, new_val| {
                s.collider_props.is_sensor = new_val;
                handle_or_return!(s, handle);
                mut_world_or_return!(s, world);
                world.collider_set[handle].set_sensor(new_val);
            })
            .done();
    };
}

// bridges common props towards collider builder.
macro_rules! complete_collider {
    ($self:ident,$base:ident,$ppm:ident,$builder:expr) => {{
        use rapier2d::prelude::*;
        $builder
            .translation(vector![
                $base.position().x / $ppm.0,
                $base.position().y / $ppm.0
            ])
            .rotation($base.rotation() as f32)
            .friction($self.collider_props.friction)
            .density($self.collider_props.density)
            .restitution($self.collider_props.restitution)
            .sensor($self.collider_props.is_sensor)
            .build()
    }};
}

// --------------------------------
// Circle

#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_properties)]
pub struct GodotRapier2DColliderBall {
    world_props: WorldProps,
    collider_props: ColliderProps,
    radius: f32,
}
proxy_world_awareness_to_world_props!(GodotRapier2DColliderBall);

#[methods]
impl GodotRapier2DColliderBall {
    pub fn new(_base: &Node2D) -> Self {
        Self {
            world_props: WorldProps::default(),
            collider_props: ColliderProps::default(),
            radius: 0.5,
        }
    }

    pub fn build_collider(&self, base: TRef<'_, Node2D>, ppm: PixelsPerMeter) -> r2d::Collider {
        complete_collider!(self, base, ppm, r2d::ColliderBuilder::ball(self.radius))
    }

    fn register_properties(builder: &ClassBuilder<GodotRapier2DColliderBall>) {
        builder
            .property("radius")
            .with_default(0.5)
            .with_getter(|s, _| s.radius)
            .with_setter(|s, b, new_val| {
                s.radius = new_val;
                b.update();

                handle_or_return!(s, handle);
                mut_world_or_return!(s, world);
                let Some(sh) = world.collider_set[handle].shape_mut().as_ball_mut() else {return};

                sh.radius = new_val
            })
            .done();
        proxy_properties_to_collider_props!(builder);
    }

    #[method]
    pub fn _draw(&self, #[base] base: TRef<'_, Node2D>) {
        if should_draw!(base) {
            let Some(ppm) = self.world_props.world_ppm else {return};
            base.draw_circle(
                Vector2 { x: 0., y: 0. },
                (self.radius * ppm.0) as f64,
                shape_color!(),
            );
        }
    }
}

// --------------------------------
// Cuboid

#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_properties)]
pub struct GodotRapier2DColliderCuboid {
    world_props: WorldProps,
    collider_props: ColliderProps,
    half_extents: Vector2,
}
proxy_world_awareness_to_world_props!(GodotRapier2DColliderCuboid);
#[methods]
impl GodotRapier2DColliderCuboid {
    pub fn new(_base: &Node2D) -> Self {
        Self {
            collider_props: ColliderProps::default(),
            world_props: WorldProps::default(),
            half_extents: Vector2 { x: 0.5, y: 0.5 },
        }
    }

    pub fn build_collider(&self, base: TRef<'_, Node2D>, ppm: PixelsPerMeter) -> r2d::Collider {
        complete_collider!(
            self,
            base,
            ppm,
            ColliderBuilder::cuboid(self.half_extents.x, self.half_extents.y)
        )
    }

    fn register_properties(builder: &ClassBuilder<GodotRapier2DColliderCuboid>) {
        builder
            .property("half_extents")
            .with_default(Vector2 { x: 0.5, y: 0.5 })
            .with_getter(|s, _| s.half_extents)
            .with_setter(|s, b, new_val| {
                s.half_extents = new_val;
                b.update();
                handle_or_return!(s,handle);
                mut_world_or_return!(s,world);
                let Some(cuboid) = world.collider_set[handle].shape_mut().as_cuboid_mut() else {return};

                use rapier2d::prelude::*;
                cuboid.half_extents = vector![new_val.x, new_val.y];
            })
            .done();
        proxy_properties_to_collider_props!(builder);
    }

    #[method]
    pub fn _draw(&self, #[base] base: TRef<'_, Node2D>) {
        if should_draw!(base) {
            let Some(ppm) = self.world_props.world_ppm else {return};
            base.draw_rect(
                Rect2 {
                    position: Vector2 {
                        x: -self.half_extents.x * ppm.0,
                        y: -self.half_extents.y * ppm.0,
                    },
                    size: Vector2 {
                        x: self.half_extents.x * 2.0 * ppm.0,
                        y: self.half_extents.y * 2.0 * ppm.0,
                    },
                },
                shape_color!(),
                true,
                0.0,
                false,
            );
        }
    }
}
