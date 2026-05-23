//! World-space rigid-body pose kept in [`f64`] for stable simulation I/O beside [`Transform`].

#[cfg(feature = "dim2")]
use bevy::math::DVec2;
#[cfg(feature = "dim3")]
use bevy::math::{DQuat, DVec3};
use bevy::prelude::*;
use rapier::math::Pose;

use crate::math::Real;
#[cfg(feature = "dim2")]
use crate::math::Vect;
#[cfg(feature = "dim3")]
use crate::math::AsPrecise;

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
#[cfg(feature = "dim2")]
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
    /// Builds a Rapier [`Pose`] from this pose, narrowing translation and rotation
    /// from `f64` to [`Real`] when the active build is `f32`.
    pub fn to_pose(self) -> Pose {
        Pose::new(
            Vect::new(
                real_from_storage(self.translation.x),
                real_from_storage(self.translation.y),
            ),
            real_from_storage(self.rotation),
        )
    }

    /// Builds a [`PhysicsTransform`] from a Rapier [`Pose`], widening from [`Real`]
    /// to `f64` when the active build is `f32`.
    pub fn from_pose(pose: &Pose) -> Self {
        Self {
            translation: DVec2::new(
                storage_from_real(pose.translation.x),
                storage_from_real(pose.translation.y),
            ),
            rotation: storage_from_real(pose.rotation.angle()),
        }
    }
}

#[cfg(feature = "dim3")]
impl PhysicsTransform {
    /// Builds a Rapier [`Pose`] from this pose, narrowing translation and rotation
    /// from `f64` to [`Real`] when the active build is `f32`.
    pub fn to_pose(self) -> Pose {
        Pose::from_parts(self.translation.as_precise(), self.rotation.as_precise())
    }

    /// Builds a [`PhysicsTransform`] from a Rapier [`Pose`], widening from [`Real`]
    /// to `f64` when the active build is `f32`.
    pub fn from_pose(pose: &Pose) -> Self {
        let t = pose.translation;
        let r = pose.rotation;
        Self {
            translation: DVec3::new(
                storage_from_real(t.x),
                storage_from_real(t.y),
                storage_from_real(t.z),
            ),
            rotation: DQuat::from_xyzw(
                storage_from_real(r.x),
                storage_from_real(r.y),
                storage_from_real(r.z),
                storage_from_real(r.w),
            ),
        }
    }
}

impl From<PhysicsTransform> for Pose {
    fn from(pt: PhysicsTransform) -> Self {
        pt.to_pose()
    }
}

impl From<&Pose> for PhysicsTransform {
    fn from(pose: &Pose) -> Self {
        Self::from_pose(pose)
    }
}
