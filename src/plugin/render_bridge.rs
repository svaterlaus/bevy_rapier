//! Optional render bridge: syncs [`PhysicsTransform`] → [`Transform`] each frame after Rapier
//! writeback, subtracting a [`RenderOrigin`] in f64 before narrowing to f32.
//!
//! Add [`PhysicsTransformRenderBridgePlugin`] to the app to opt in. The plugin is **not** included
//! in [`super::RapierPhysicsPlugin`]'s default set so existing users see no behaviour change.

use crate::dynamics::PhysicsTransform;
use crate::plugin::PhysicsSet;
use bevy::prelude::*;

#[cfg(feature = "dim2")]
use bevy::math::DVec2;
#[cfg(feature = "dim3")]
use bevy::math::DVec3;

/// World-space origin used to re-centre all [`PhysicsTransform`] positions before writing to
/// [`Transform`].
///
/// Set this each frame to the camera's position in physics space (a `DVec2`/`DVec3`).  The bridge
/// system subtracts `RenderOrigin` from each body's `PhysicsTransform.translation` in f64, then
/// narrows the small residual to f32, keeping every entity within a bounded window around the
/// camera where f32 precision is adequate.
///
/// Defaults to zero (world origin). Mutation is intentionally left to game code — a typical use is
/// a one-line system that copies the camera's `PhysicsTransform.translation` into this resource
/// each `PostUpdate` frame.
#[derive(Resource, Debug, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct RenderOrigin(
    #[cfg(feature = "dim2")] pub DVec2,
    #[cfg(feature = "dim3")] pub DVec3,
);

impl Default for RenderOrigin {
    fn default() -> Self {
        #[cfg(feature = "dim2")]
        return Self(DVec2::ZERO);
        #[cfg(feature = "dim3")]
        return Self(DVec3::ZERO);
    }
}

/// Opt-in plugin that drives [`Transform`] from [`PhysicsTransform`] relative to [`RenderOrigin`].
///
/// Runs one system in `PostUpdate` after [`PhysicsSet::Writeback`] and before
/// [`bevy::transform::TransformSystems::Propagate`].  The system writes every entity that has both
/// a [`PhysicsTransform`] and a [`Transform`], so entities without `PhysicsTransform` are unaffected.
///
/// Add this plugin **after** [`super::RapierPhysicsPlugin`]:
///
/// ```rust,ignore
/// app.add_plugins((
///     RapierPhysicsPlugin::<NoUserData>::default(),
///     PhysicsTransformRenderBridgePlugin,
/// ));
/// ```
pub struct PhysicsTransformRenderBridgePlugin;

impl Plugin for PhysicsTransformRenderBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderOrigin>()
            .register_type::<RenderOrigin>()
            .add_systems(
                PostUpdate,
                sync_physics_transform_to_transform
                    .after(PhysicsSet::Writeback)
                    .before(bevy::transform::TransformSystems::Propagate),
            );
    }
}

/// Writes `(PhysicsTransform.translation - RenderOrigin)` into `Transform.translation`, narrowing
/// from f64 to f32 only after the subtraction so the residual is always small.
///
/// The z-component of `Transform.translation` is preserved, allowing sprite layering to coexist
/// with the bridge without requiring a separate system.
pub fn sync_physics_transform_to_transform(
    origin: Res<RenderOrigin>,
    mut q: Query<(&PhysicsTransform, &mut Transform)>,
) {
    for (pt, mut t) in &mut q {
        // Subtract in f64 first; only the small offset from origin enters f32.
        #[cfg(feature = "dim2")]
        {
            let rel = (pt.translation - origin.0).as_vec2();
            t.translation.x = rel.x;
            t.translation.y = rel.y;
            t.rotation = Quat::from_rotation_z(pt.rotation as f32);
        }
        #[cfg(feature = "dim3")]
        {
            let rel = (pt.translation - origin.0).as_vec3();
            t.translation = rel;
            t.rotation = pt.rotation.as_quat();
        }
    }
}
