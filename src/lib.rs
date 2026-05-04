//!
//! # Official integration of Rapier to the Bevy game engine
//!
//! Rapier is a set of two Rust crates `rapier2d` and `rapier3d` for efficient cross-platform
//! physics simulation. Its target application include video games, animation, robotics, etc.
//!
//! The `bevy_rapier` project implements two crates `bevy_rapier2d` and `bevy_rapier3d` which
//! define physics plugins for the Bevy game engine.
//!
//! User documentation for `bevy_rapier` is on [the official Rapier site](https://rapier.rs/docs/).
//!
#![warn(missing_docs)]

#[macro_use]
#[cfg(feature = "serde-serialize")]
extern crate serde;

pub extern crate nalgebra as na;
#[cfg(all(feature = "dim2", feature = "f32"))]
pub extern crate rapier2d as rapier;
#[cfg(all(feature = "dim2", feature = "f64"))]
pub extern crate rapier2d_f64 as rapier;

#[cfg(all(feature = "dim3", feature = "f32"))]
pub extern crate rapier3d as rapier;
#[cfg(all(feature = "dim3", feature = "f64"))]
pub extern crate rapier3d_f64 as rapier;

pub use rapier::parry;

/// Type aliases to select the right vector / rotation types based
/// on the dimension and scalar precision selected by Cargo features for this crate build.
#[cfg(feature = "dim2")]
pub mod math {
    pub use crate::utils::as_precise::*;

    /// Scalar type backing Rapier simulation values (`f32` or `f64`).
    pub type Real = rapier::math::Real;
    /// The vector type matching [`Real`].
    #[cfg(feature = "f32")]
    pub type Vect = bevy::math::Vec2;
    /// The vector type matching [`Real`].
    #[cfg(feature = "f64")]
    pub type Vect = bevy::math::DVec2;

    /// Integer grid offsets in the physics plane (`i32` components).
    pub type IVect = bevy::math::IVec2;
    /// 2D rotation as a scalar angle (radians), matching Rapier representation.
    pub type Rot = Real;
}

/// Type aliases as in [`crate::math`], but for 3D.
#[cfg(feature = "dim3")]
pub mod math {
    pub use crate::utils::as_precise::*;

    /// Scalar type backing Rapier simulation values (`f32` or `f64`).
    pub type Real = rapier::math::Real;
    /// The vector type matching [`Real`].
    #[cfg(feature = "f32")]
    pub type Vect = bevy::math::Vec3;
    /// The vector type matching [`Real`].
    #[cfg(feature = "f64")]
    pub type Vect = bevy::math::DVec3;

    /// Integer grid offsets in the physics volume (`i32` components).
    pub type IVect = bevy::math::IVec3;
    /// Unit quaternion rotation matching Rapier representation.
    #[cfg(feature = "f32")]
    pub type Rot = bevy::math::Quat;
    /// Double-precision unit quaternion matching Rapier representation.
    #[cfg(feature = "f64")]
    pub type Rot = bevy::math::DQuat;
}

/// Components related to physics dynamics (rigid-bodies, velocities, etc.)
pub mod dynamics;
/// Components related to physics geometry (colliders, collision-groups, etc.)
pub mod geometry;
/// Components and resources related to the physics simulation workflow (events, hooks, etc.)
pub mod pipeline;
/// The physics plugin and systems.
pub mod plugin;

/// Reflection utilities.
pub mod reflect;

#[cfg(feature = "picking-backend")]
pub mod picking_backend;

/// Components related to character control.
pub mod control;
/// The debug-renderer.
#[cfg(any(feature = "debug-render-3d", feature = "debug-render-2d"))]
pub mod render;
/// Miscellaneous helper functions.
pub mod utils;

/// Groups the most often used types.
pub mod prelude {
    pub use crate::control::*;
    pub use crate::dynamics::*;
    pub use crate::geometry::*;
    pub use crate::math::*;
    #[cfg(feature = "picking-backend")]
    pub use crate::picking_backend::*;
    pub use crate::pipeline::*;
    pub use crate::plugin::context::systemparams::*;
    pub use crate::plugin::context::*;
    pub use crate::plugin::*;
    #[cfg(any(feature = "debug-render-3d", feature = "debug-render-2d"))]
    pub use crate::render::*;
}
