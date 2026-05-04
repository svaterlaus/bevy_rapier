//! World-space rigid-body pose kept in [`f64`] for stable simulation I/O beside [`Transform`].

#[cfg(feature = "dim2")]
use bevy::math::DVec2;
#[cfg(feature = "dim3")]
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use rapier::math::Isometry;

use crate::math::Real;

/// World-space rigid-body pose in double precision (`f64`), independent of [`Transform`].
///
/// When [`crate::plugin::configuration::PhysicsTransformRouting::PhysicsTransform`] is active for a
/// context, Rapier pose sync flows through this component instead of [`GlobalTransform`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Reflect)]
#[reflect(Component, Default, PartialEq)]
pub struct PhysicsTransform {
    /// World-space translation (`x`,`y`; `z` is carried on [`Transform`] when needed for layering).
    #[cfg(feature = "dim2")]
    pub translation: DVec2,
    /// Translation in Rapier world space.
    #[cfg(feature = "dim3")]
    pub translation: DVec3,
    /// Rotation angle in radians (Rapier 2D convention).
    #[cfg(feature = "dim2")]
    pub rotation: f64,
    /// Orientation as a normalized quaternion in world space.
    #[cfg(feature = "dim3")]
    pub rotation: DQuat,
}

#[inline]
fn real_from_storage(x: f64) -> Real {
    #[cfg(feature = "f64")]
    {
        x
    }
    #[cfg(feature = "f32")]
    {
        Real::from(x as f32)
    }
}

#[inline]
fn storage_from_real(r: Real) -> f64 {
    #[cfg(feature = "f64")]
    {
        r
    }
    #[cfg(feature = "f32")]
    {
        r as f64
    }
}

#[cfg(feature = "dim2")]
impl PhysicsTransform {
    pub(crate) fn to_isometry(self) -> Isometry<Real> {
        Isometry::<Real>::new(
            crate::na::Vector2::new(
                real_from_storage(self.translation.x),
                real_from_storage(self.translation.y),
            ),
            real_from_storage(self.rotation),
        )
    }

    pub(crate) fn from_isometry(iso: &Isometry<Real>) -> Self {
        let v = iso.translation.vector;
        Self {
            translation: DVec2::new(storage_from_real(v.x), storage_from_real(v.y)),
            rotation: storage_from_real(iso.rotation.angle()),
        }
    }
}

#[cfg(feature = "dim3")]
impl PhysicsTransform {
    pub(crate) fn to_isometry(self) -> Isometry<Real> {
        use crate::na::{Isometry3, Quaternion as NaQuat, Translation3, UnitQuaternion, Vector3};

        Isometry3::from_parts(
            Translation3::new(
                real_from_storage(self.translation.x),
                real_from_storage(self.translation.y),
                real_from_storage(self.translation.z),
            ),
            UnitQuaternion::new_normalize(NaQuat::from_parts(
                real_from_storage(self.rotation.w),
                Vector3::new(
                    real_from_storage(self.rotation.x),
                    real_from_storage(self.rotation.y),
                    real_from_storage(self.rotation.z),
                ),
            )),
        )
    }

    pub(crate) fn from_isometry(iso: &Isometry<Real>) -> Self {
        let t_vec = iso.translation.vector;
        let rq: crate::na::UnitQuaternion<Real> = iso.rotation;
        let q = rq.quaternion();
        Self {
            translation: DVec3::new(
                storage_from_real(t_vec.x),
                storage_from_real(t_vec.y),
                storage_from_real(t_vec.z),
            ),
            rotation: DQuat::from_xyzw(
                storage_from_real(q.vector()[0]),
                storage_from_real(q.vector()[1]),
                storage_from_real(q.vector()[2]),
                storage_from_real(q.scalar()),
            ),
        }
    }
}
