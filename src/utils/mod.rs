/// Widening and narrowing helpers between Bevy single precision and Rapier’s active [`crate::math::Real`].
pub mod as_precise;

use self::as_precise::AsSingle;
use crate::math::Real;
use bevy::prelude::{Quat, Transform, Vec3};
use rapier::math::Isometry;

/// Converts a Rapier isometry to a [`Transform`] (always `f32`; may lose magnitude far from origin).
#[cfg(feature = "dim2")]
pub fn iso_to_transform(iso: &Isometry<Real>) -> Transform {
    let v = &iso.translation.vector;
    Transform {
        translation: Vec3::new(v.x.as_single(), v.y.as_single(), 0.0),
        rotation: Quat::from_rotation_z(iso.rotation.angle().as_single()),
        ..Default::default()
    }
}

/// Converts a Rapier isometry to a [`Transform`].
#[cfg(feature = "dim3")]
pub fn iso_to_transform(iso: &Isometry<Real>) -> Transform {
    use crate::na::{Quaternion as NaQuat, UnitQuaternion};
    let t = iso.translation.vector;
    let rq: UnitQuaternion<Real> = iso.rotation;

    Transform {
        translation: Vec3::new(t.x.as_single(), t.y.as_single(), t.z.as_single()),
        rotation: {
            let q: NaQuat<Real> = *rq.quaternion();
            Quat::from_xyzw(
                q.vector()[0].as_single(),
                q.vector()[1].as_single(),
                q.vector()[2].as_single(),
                q.scalar().as_single(),
            )
        },
        ..Default::default()
    }
}

/// Converts a Bevy [`Transform`] into a Rapier isometry (widens `f32` components to [`Real`]).
#[cfg(feature = "dim2")]
pub(crate) fn transform_to_iso(transform: &Transform) -> Isometry<Real> {
    use bevy::math::Vec3Swizzles;
    let xy = transform.translation.xy();
    Isometry::new(
        crate::na::Vector2::new(Real::from(xy.x), Real::from(xy.y)),
        Real::from(transform.rotation.to_scaled_axis().z),
    )
}

/// Converts a Bevy [`Transform`] into a Rapier isometry.
#[cfg(feature = "dim3")]
pub(crate) fn transform_to_iso(transform: &Transform) -> Isometry<Real> {
    use crate::na::{Isometry3, Quaternion, Translation3, UnitQuaternion, Vector3};
    let t = transform.translation;
    let r = transform.rotation;
    Isometry3::from_parts(
        Translation3::new(Real::from(t.x), Real::from(t.y), Real::from(t.z)),
        UnitQuaternion::new_normalize(Quaternion::from_parts(
            Real::from(r.w),
            Vector3::new(Real::from(r.x), Real::from(r.y), Real::from(r.z)),
        )),
    )
}

#[cfg(test)]
#[cfg(feature = "dim3")]
mod tests {
    use super::*;
    use bevy::prelude::Transform;

    #[test]
    fn convert_back_to_equal_transform() {
        let transform = Transform {
            translation: bevy::prelude::Vec3::new(-2.1855694e-8, 0.0, 0.0),
            rotation: bevy::prelude::Quat::from_xyzw(0.99999994, 0.0, 1.6292068e-7, 0.0)
                .normalize(),
            ..Default::default()
        };
        let converted_transform = iso_to_transform(&transform_to_iso(&transform));
        assert_eq!(converted_transform, transform);
    }
}
