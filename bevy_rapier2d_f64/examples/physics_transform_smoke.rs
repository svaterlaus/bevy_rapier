//! Spawns one dynamic body far from the origin and logs high-precision translations each frame.
//!
//! Uses [`PhysicsTransform`] as canonical world pose (default [`PhysicsTransformRouting`] on f64 builds).
//! With fixed timestep and zero gravity, verifies linear motion `Δp = v·Δt` in f64 instead of relying
//! on quantized [`Transform`] / [`GlobalTransform`].
//!
//! Also wires [`PhysicsTransformRenderBridgePlugin`] and [`RenderOrigin`] so debug gizmos stay aligned
//! with camera-relative rendering at large world coordinates.

use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::transform::TransformSystems;
use bevy_rapier2d_f64::prelude::*;
use std::time::Duration;

const DT_RAP: f64 = 1.0 / 60.0;
const TICKS_BEFORE_DRIFT_CHECK: u32 = 120;
const TICKS_BEFORE_SUBMETER_SAMPLES: u32 = 12;

#[derive(Component)]
struct Probe;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            PhysicsTransformRenderBridgePlugin,
            RapierDebugRenderPlugin::default(),
        ))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            DT_RAP,
        )))
        .insert_resource(TimestepMode::Fixed {
            dt: DT_RAP,
            substeps: 1,
        })
        .add_systems(
            Startup,
            (
                setup_graphics,
                setup_physics,
                zero_gravity,
                assert_physics_transform_routing,
            ),
        )
        .add_systems(Update, tick_probe_logging)
        .add_systems(
            PostUpdate,
            (
                sync_render_origin_from_probe,
                drift_check,
            )
                .chain()
                .after(PhysicsSet::Writeback)
                .before(TransformSystems::Propagate),
        )
        .run();
}

fn setup_graphics(mut commands: Commands) {
    // Offset from the probe once [`RenderOrigin`] tracks it each frame.
    commands.spawn((Camera2d, Transform::from_xyz(2.0, 120.0, 0.0)));
}

fn sync_render_origin_from_probe(
    mut origin: ResMut<RenderOrigin>,
    probe: Query<&PhysicsTransform, With<Probe>>,
) {
    if let Ok(pt) = probe.single() {
        origin.position = pt.translation;
    }
}

fn zero_gravity(mut q: Query<&mut RapierConfiguration, With<DefaultRapierContext>>) {
    if let Ok(mut cfg) = q.single_mut() {
        cfg.gravity = Vect::ZERO;
    }
}

fn assert_physics_transform_routing(cfgs: Query<&RapierConfiguration, With<DefaultRapierContext>>) {
    let cfg = cfgs.single().expect("default rapier context");
    assert!(
        cfg.physics_transform_routing == PhysicsTransformRouting::PhysicsTransform,
        "physics_transform_smoke assumes PhysicsTransformRouting::PhysicsTransform (f64 default)"
    );
}

fn setup_physics(mut commands: Commands) {
    let v = DVec2::new(1.23456789012345, -0.000987654321);
    commands.spawn((
        Probe,
        PhysicsTransform {
            translation: DVec2::new(5_000_000.0, 0.0),
            rotation: 0.0,
        },
        Transform::IDENTITY,
        RigidBody::Dynamic,
        Velocity {
            linear: v,
            angular: 0.0,
        },
        Collider::ball(1.0),
    ));
}

#[derive(Default)]
struct LogTicks(u32);

fn tick_probe_logging(mut n: Local<LogTicks>, probe: Query<&PhysicsTransform, With<Probe>>) {
    if n.0 >= TICKS_BEFORE_SUBMETER_SAMPLES {
        return;
    }
    let Ok(pt) = probe.single() else {
        return;
    };
    println!(
        "[tick {}] physics_transform.translation = ({:.14}, {:.14}), frac_x = {:.14}",
        n.0,
        pt.translation.x,
        pt.translation.y,
        pt.translation.x.fract(),
    );
    n.0 += 1;
}

#[derive(Default)]
struct DriftProbe {
    ticks: u32,
    checked: bool,
}

fn drift_check(
    mut s: Local<DriftProbe>,
    probe: Query<(&PhysicsTransform, &Velocity), With<Probe>>,
) {
    if s.checked {
        return;
    }
    s.ticks += 1;
    if s.ticks < TICKS_BEFORE_DRIFT_CHECK {
        return;
    }
    let Ok((pt, vel)) = probe.single() else {
        return;
    };

    let elapsed = TICKS_BEFORE_DRIFT_CHECK as f64 * DT_RAP;
    let p0 = DVec2::new(5_000_000.0, 0.0);
    let expected_delta = vel.linear * elapsed;
    let actual_delta = pt.translation - p0;
    let err = (expected_delta - actual_delta).length();
    assert!(
        err < 1.0e-6,
        "linear drift mismatch: expected delta {expected_delta:?}, got {actual_delta:?}, error {err:e}",
    );

    println!(
        "[smoke OK] physics_transform after {} ticks matches v·Δt (err = {:.3e})",
        TICKS_BEFORE_DRIFT_CHECK, err,
    );

    s.checked = true;
}
