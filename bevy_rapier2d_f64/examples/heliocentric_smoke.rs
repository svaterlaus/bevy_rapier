//! Spawns one dynamic body far from the origin and logs Rapier’s internal translation each frame.
//!
//! Intended as a coarse smoke signal that `bevy_rapier2d_f64` preserves sub-meter detail in the solver
//! at heliocentric scales (Bevy `Transform` writes remain f32-quantized; see Phase C).

use bevy::math::DVec2;
use bevy::prelude::*;
use bevy_rapier2d_f64::prelude::*;

#[derive(Component)]
struct HelioProbe;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .add_systems(Startup, (setup_graphics, setup_physics, zero_gravity))
        .add_systems(Update, log_probe_translation)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(5_000_002.0, 120.0, 0.0)));
}

fn zero_gravity(mut q: Query<&mut RapierConfiguration, With<DefaultRapierContext>>) {
    if let Ok(mut cfg) = q.single_mut() {
        cfg.gravity = Vect::ZERO;
    }
}

fn setup_physics(mut commands: Commands) {
    commands.spawn((
        HelioProbe,
        Transform::from_xyz(5_000_000.0_f32, 0.0, 0.0),
        RigidBody::Dynamic,
        Velocity {
            linvel: DVec2::new(1.23456789012345, -0.000987654321),
            angvel: 0.0,
        },
        Collider::ball(1.0),
    ));
}

fn log_probe_translation(
    mut n: Local<u32>,
    probe: Query<&RapierRigidBodyHandle, With<HelioProbe>>,
    bodies: Query<&RapierRigidBodySet, With<DefaultRapierContext>>,
) {
    if *n >= 12 {
        return;
    }
    let Ok(handle) = probe.single() else {
        return;
    };
    let Ok(rb_set) = bodies.single() else {
        return;
    };
    let Some(rb) = rb_set.bodies.get(handle.0) else {
        return;
    };
    let t = rb.translation();
    println!(
        "[frame {}] rapier.translation = ({:.14}, {:.14})",
        *n, t.x, t.y,
    );
    *n += 1;
}
