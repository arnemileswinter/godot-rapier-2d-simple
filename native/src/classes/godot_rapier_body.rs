use std::{cell::RefCell, rc::Weak};

use gdnative::{export::hint::EnumHint, prelude::*};
use rapier2d::prelude::{self as r2d, RigidBodyBuilder};

use super::godot_rapier_world::WorldState;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RapierRigidBodyType(r2d::RigidBodyType);
impl ToVariant for RapierRigidBodyType {
    fn to_variant(&self) -> Variant {
        Variant::new(self.0 as isize)
    }
}
impl FromVariant for RapierRigidBodyType {
    fn from_variant(variant: &Variant) -> Result<Self, FromVariantError> {
        let i = isize::from_variant(variant)?;
        let bt = match i {
            0 => Ok(r2d::RigidBodyType::Dynamic),
            1 => Ok(r2d::RigidBodyType::Fixed),
            2 => Ok(r2d::RigidBodyType::KinematicPositionBased),
            3 => Ok(r2d::RigidBodyType::KinematicVelocityBased),
            _ => Err(FromVariantError::Custom(format!(
                "no rigid body type with index {}!",
                i
            ))),
        }?;
        Ok(RapierRigidBodyType(bt))
    }
}
impl gdnative::export::Export for RapierRigidBodyType {
    type Hint = gdnative::export::hint::IntHint<isize>;

    fn export_info(hint: Option<Self::Hint>) -> ExportInfo {
        hint.map_or_else(
            || {
                gdnative::export::hint::IntHint::<isize>::Enum(EnumHint::new(vec![
                    "Dynamic".into(),
                    "Fixed".into(),
                    "KinematicPositionBased".into(),
                    "KinematicVelocityBased".into(),
                ]))
                .export_info()
            },
            Self::Hint::export_info,
        )
    }
}

#[derive(NativeClass)]
#[inherit(Node2D)]
pub struct GodotRapierBody {
    pub world_ref: Option<Weak<RefCell<WorldState>>>,
    pub handle: Option<r2d::RigidBodyHandle>,
    #[property]
    pub rigid_body_type: RapierRigidBodyType,
}

impl GodotRapierBody {
    pub fn build_rigid_body(&self, _base: TRef<'_, Node2D>) -> r2d::RigidBody {
        let b = RigidBodyBuilder::new(self.rigid_body_type.0);
        b.build()
    }
}

#[methods]
impl GodotRapierBody {
    fn new(_base: &Node2D) -> Self {
        GodotRapierBody {
            world_ref: None,
            handle: None,
            rigid_body_type: RapierRigidBodyType(r2d::RigidBodyType::Dynamic),
        }
    }

    #[method]
    fn _physics_process(&mut self, #[base] base: &Node2D, _delta: f32) {
        let Some(world_ref) = &self.world_ref else {return};
        let Some(handle) = self.handle else {return};
        let Some(world_ref) = world_ref.upgrade() else {
            self.handle = None;
            return
        };
        let world = (*world_ref).borrow();
        let Some(body) = world.rigid_body_set.get(handle) else {return};
        let iso = body.position();

        // copy rapier isometry onto godot object.
        base.set_global_position(Vector2 {
            x: iso.translation.x,
            y: iso.translation.y,
        });
        base.set_global_rotation(iso.rotation.angle() as f64)
    }
}
