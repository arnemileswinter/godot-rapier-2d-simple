use std::cell::RefCell;
use std::rc::Rc;

use gdnative::export::hint::{FloatHint, RangeHint};
use gdnative::prelude::*;

use crate::classes::godot_rapier_2d_body::{
    GodotRapier2DDynamicBody, GodotRapier2DFixedBody, GodotRapier2DKinematicPositionBasedBody,
    GodotRapier2DKinematicVelocityBasedBody,
};

use super::common::{not_in_editor, PixelsPerMeter, World2DAware, World2DState};

const G: f32 = 9.81;

#[derive(NativeClass)]
#[inherit(Node2D)]
#[register_with(Self::register_properties)]
pub struct GodotRapier2DWorld {
    rapier_world: Rc<RefCell<World2DState>>,
    // propery
    pixels_per_meter: PixelsPerMeter,
    // property
    gravity: Vector2,
}

impl GodotRapier2DWorld {}

#[methods]
impl GodotRapier2DWorld {
    fn new(_base: &Node2D) -> Self {
        let mut w = World2DState::default();
        w.gravity.y = G;
        Self {
            rapier_world: Rc::new(RefCell::new(w)),
            pixels_per_meter: PixelsPerMeter(64.),
            gravity: Vector2 { x: 0., y: G },
        }
    }

    fn register_properties(builder: &ClassBuilder<GodotRapier2DWorld>) {
        builder
            .property("pixels_per_meter")
            .with_default(64.)
            .with_getter(|s, _| s.pixels_per_meter.0)
            .with_setter(|s, _, new_val| s.pixels_per_meter = PixelsPerMeter(new_val))
            .with_hint(FloatHint::Range(
                RangeHint::new(1.0, 64.0).or_greater().with_step(1.),
            ))
            .done();
        builder
            .property("gravity")
            .with_default(Vector2 { x: 0., y: G })
            .with_getter(|s, _| s.gravity)
            .with_setter(|s, _, new_val| {
                use rapier2d::prelude::*;
                s.gravity = new_val;
                s.rapier_world.borrow_mut().gravity = vector![new_val.x, new_val.y];
            })
            .done();
    }

    #[method]
    fn _ready(&mut self, #[base] base: TRef<'_, Node2D>) {
        base.get_children().iter().for_each(|c| {
            let Ok(n) = c.try_to() else {return;};
            self._register_child(base, n);
        })
    }

    #[method]
    fn _physics_process(&mut self, #[base] _base: TRef<'_, Node2D>, _delta: f64) {
        not_in_editor!();
        self.rapier_world.borrow_mut().tick();
    }

    #[method]
    fn add_child(
        &mut self,
        #[base] base: TRef<'_, Node2D>,
        node: Ref<Node>
    ) {
        base.add_child(node, false);
        self._register_child(base, node)
    }

    #[method]
    fn remove_child(&mut self, #[base] base: TRef<'_, Node2D>, node: Ref<Node>) {
        self._unregister_child(base, node);
        base.remove_child(node);
    }

    #[method]
    fn _register_child(&mut self, #[base] base: TRef<'_, Node2D>, node: Ref<Node>) {
        let Some(n) : Option<TRef<'_,Node2D>> = unsafe { node.assume_safe() }.cast() else {return};

        macro_rules! register_child {
            ($i:ident) => {{
                $i.map_mut(move |n, b| {
                    let handle = self
                        .rapier_world
                        .borrow_mut()
                        .rigid_body_set
                        .insert(n.build_rigid_body(b, self.pixels_per_meter));
                    n.add_to_world(b, &self.rapier_world, handle, self.pixels_per_meter)
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
            }};
        }

        if let Some(i) = n.cast_instance::<GodotRapier2DFixedBody>() {
            register_child!(i)
        } else if let Some(i) = n.cast_instance::<GodotRapier2DDynamicBody>() {
            register_child!(i)
        } else if let Some(i) = n.cast_instance::<GodotRapier2DKinematicPositionBasedBody>() {
            register_child!(i)
        } else if let Some(i) = n.cast_instance::<GodotRapier2DKinematicVelocityBasedBody>() {
            register_child!(i)
        }
    }

    #[method]
    fn _unregister_child(&mut self, #[base] _base: TRef<'_, Node2D>, node: Ref<Node>) {
        let Some(n) : Option<TRef<'_,Node2D>> = unsafe { node.assume_safe() }.cast() else {return};

        macro_rules! unregister_child {
            ($i:ident) => {
                $i.map_mut(|n, b| n.remove_from_world(b)).unwrap()
            };
        }

        if let Some(i) = n.cast_instance::<GodotRapier2DFixedBody>() {
            unregister_child!(i)
        } else if let Some(i) = n.cast_instance::<GodotRapier2DDynamicBody>() {
            unregister_child!(i)
        } else if let Some(i) = n.cast_instance::<GodotRapier2DKinematicPositionBasedBody>() {
            unregister_child!(i)
        } else if let Some(i) = n.cast_instance::<GodotRapier2DKinematicVelocityBasedBody>() {
            unregister_child!(i)
        }
    }
}
