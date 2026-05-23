//! Applies and writes back [`crate::dynamics::PhysicsTransform`] when it is selected in [`crate::plugin::configuration::PhysicsTransformRouting`].

use crate::dynamics::{PhysicsTransform, RapierRigidBodyHandle, TransformInterpolation};
use crate::plugin::configuration::{PhysicsTransformRouting, RapierConfiguration, TimestepMode};
use crate::plugin::context::systemparams::RAPIER_CONTEXT_EXPECT_ERROR;
use crate::plugin::context::{RapierContextEntityLink, RapierRigidBodySet, SimulationToRenderTime};
use crate::prelude::{RigidBody, RigidBodyDisabled};
use bevy::prelude::*;
use rapier::dynamics::{RigidBodyHandle, RigidBodyType};
use std::collections::HashMap;

/// System applying user-authored [`PhysicsTransform`] changes into Rapier rigid-body poses.
///
/// Mirrors [`crate::plugin::systems::apply_rigid_body_user_changes`]’s [`GlobalTransform`] path.
pub fn apply_physics_transform_user_changes(
    mut rigid_body_sets: Query<&mut RapierRigidBodySet>,
    config: Query<&RapierConfiguration>,
    mut changed_pts: Query<
        (
            &RapierRigidBodyHandle,
            &RapierContextEntityLink,
            &PhysicsTransform,
            Option<&mut TransformInterpolation>,
        ),
        Changed<PhysicsTransform>,
    >,
) {
    let physics_transform_changed_fn =
        |handle: &RigidBodyHandle,
         cfg: &RapierConfiguration,
         pt: &PhysicsTransform,
         last_set: &HashMap<RigidBodyHandle, PhysicsTransform>| {
            if cfg.force_update_from_transform_changes {
                true
            } else if let Some(prev) = last_set.get(handle) {
                *prev != *pt
            } else {
                true
            }
        };

    for (handle, link, physics_transform, mut interpolation) in changed_pts.iter_mut() {
        let Ok(cfg) = config.get(link.0) else {
            continue;
        };
        if cfg.physics_transform_routing != PhysicsTransformRouting::PhysicsTransform {
            continue;
        }

        let rigidbody_set = rigid_body_sets
            .get_mut(link.0)
            .expect(RAPIER_CONTEXT_EXPECT_ERROR)
            .into_inner();
        let mut physics_changed = None;

        if let Some(interpolation) = interpolation.as_deref_mut() {
            physics_changed = physics_changed.or_else(|| {
                Some(physics_transform_changed_fn(
                    &handle.0,
                    cfg,
                    physics_transform,
                    &rigidbody_set.last_body_physics_transform_set,
                ))
            });

            if physics_changed == Some(true) {
                interpolation.start = None;
                interpolation.end = None;
            }
        }
        if let Some(rb) = rigidbody_set.bodies.get_mut(handle.0) {
            physics_changed = physics_changed.or_else(|| {
                Some(physics_transform_changed_fn(
                    &handle.0,
                    cfg,
                    physics_transform,
                    &rigidbody_set.last_body_physics_transform_set,
                ))
            });

            match rb.body_type() {
                RigidBodyType::KinematicPositionBased => {
                    if physics_changed == Some(true) {
                        rb.set_next_kinematic_position(physics_transform.to_pose());
                        rigidbody_set
                            .last_body_physics_transform_set
                            .insert(handle.0, *physics_transform);
                    }
                }
                _ => {
                    if physics_changed == Some(true) {
                        rb.set_position(physics_transform.to_pose(), true);
                        rigidbody_set
                            .last_body_physics_transform_set
                            .insert(handle.0, *physics_transform);
                    }
                }
            }
        }
    }
}

/// Writes simulated rigid-body poses into [`PhysicsTransform`].
pub fn writeback_physics_transform(
    mut rigid_body_sets: Query<&mut RapierRigidBodySet>,
    timestep_mode: Res<TimestepMode>,
    config: Query<&RapierConfiguration>,
    sim_to_render_time: Query<&SimulationToRenderTime>,
    mut query: Query<
        (
            &RapierRigidBodyHandle,
            &RapierContextEntityLink,
            &mut PhysicsTransform,
            Option<&mut TransformInterpolation>,
        ),
        (With<RigidBody>, Without<RigidBodyDisabled>),
    >,
) {
    for (handle, link, mut physics_transform, mut interpolation) in query.iter_mut() {
        let cfg = config
            .get(link.0)
            .expect("Could not get `RapierConfiguration`");

        if !cfg.physics_pipeline_active
            || cfg.physics_transform_routing != PhysicsTransformRouting::PhysicsTransform
        {
            continue;
        }

        let handle = handle.0;

        let rigid_body_set = rigid_body_sets
            .get_mut(link.0)
            .expect(RAPIER_CONTEXT_EXPECT_ERROR)
            .into_inner();
        let sim_to_render_time = sim_to_render_time
            .get(link.0)
            .expect("Could not get `SimulationToRenderTime`");

        if let Some(rb) = rigid_body_set.bodies.get(handle) {
            let mut interpolated_iso = *rb.position();

            if let TimestepMode::Interpolated { dt, .. } = *timestep_mode {
                if let Some(interpolation) = interpolation.as_deref_mut() {
                    if interpolation.end.is_none() {
                        interpolation.end = Some(*rb.position());
                    }

                    if let Some(interpolated) =
                        interpolation.lerp_slerp((dt + sim_to_render_time.diff) / dt)
                    {
                        interpolated_iso = interpolated;
                    }
                }
            }

            let new_pt = PhysicsTransform::from_pose(&interpolated_iso);

            if *physics_transform != new_pt {
                *physics_transform = new_pt;
            }

            rigid_body_set
                .last_body_physics_transform_set
                .insert(handle, new_pt);
        }
    }
}

#[cfg(all(test, feature = "f64", feature = "dim2"))]
mod tests {
    use crate::plugin::{NoUserData, RapierPhysicsPlugin};
    use crate::prelude::*;
    use bevy::math::DVec2;
    use bevy::prelude::{App, Component, Entity, Query, Transform, Update, Vec3, With, World};
    use bevy::time::{TimePlugin, TimeUpdateStrategy};
    use bevy::transform::TransformPlugin;
    use std::time::Duration;

    #[derive(Component)]
    struct JitterTarget;

    fn jitter_local_transform(mut q: Query<&mut Transform, With<JitterTarget>>) {
        for mut t in &mut q {
            t.translation = Vec3::new(
                (t.translation.x * 0.017 + 13.7).sin() * 1_000_000.0,
                (t.translation.y * 0.023 - 4.2).cos() * 888_888.0,
                t.translation.z,
            );
        }
    }

    fn rigid_body_translation_xy(world: &mut World) -> (f64, f64) {
        let target = world
            .query_filtered::<Entity, With<JitterTarget>>()
            .single(world)
            .expect("jitter target");
        let handle = *world
            .entity(target)
            .get::<RapierRigidBodyHandle>()
            .expect("rapier handle");

        let rigidbody_set = world
            .query::<&RapierRigidBodySet>()
            .single(world)
            .expect("rigid body set");

        let rb = rigidbody_set
            .bodies
            .get(handle.0)
            .expect("rapier rigid body");
        let t = rb.translation();
        (t.x, t.y)
    }

    fn sample_after_updates(jitter: bool) -> (f64, f64) {
        let mut app = App::new();
        app.add_plugins((
            TransformPlugin,
            TimePlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
        ));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )));
        app.insert_resource(TimestepMode::Fixed {
            dt: 1.0 / 60.0,
            substeps: 1,
        });
        if jitter {
            app.add_systems(Update, jitter_local_transform);
        }
        app.finish();

        app.world_mut().spawn((
            JitterTarget,
            PhysicsTransform {
                translation: DVec2::new(5_000_000.0, -3_141.59),
                rotation: 0.0,
            },
            Transform::IDENTITY,
            RigidBody::Dynamic,
            Velocity {
                linear: DVec2::new(-200.45, 80.125),
                angular: 0.0,
            },
            Collider::ball(0.25),
        ));

        for _ in 0..60 {
            app.update();
        }

        rigid_body_translation_xy(app.world_mut())
    }

    #[test]
    fn jittering_local_transform_does_not_shift_rapier_pose() {
        let baseline = sample_after_updates(false);
        let with_jitter = sample_after_updates(true);
        assert_eq!(
            baseline, with_jitter,
            "Rapier internal translation should ignore local Transform noise when using PhysicsTransform routing"
        );
    }
}
