use bevy::math::*;

/// Converts math types into the scalar precision selected by crate features (`f32` / `f64`).
pub trait AsPrecise: Clone + Copy {
    /// Output type at the active precision.
    type Out;
    /// Convert into the active precision.
    fn as_precise(self) -> Self::Out;
}

/// Converts high-precision types into single-precision Bevy / rendering types.
pub trait AsSingle: Clone + Copy {
    /// Single-precision output type.
    type Out;
    /// Converts to [`Self::Out`] for Transform and rendering APIs.
    fn as_single(self) -> Self::Out;
}

macro_rules! as_precise_self {
    ($ty:ty) => {
        impl AsPrecise for $ty {
            type Out = $ty;
            fn as_precise(self) -> Self::Out {
                self
            }
        }
    };
}

macro_rules! as_single_self {
    ($ty:ty) => {
        impl AsSingle for $ty {
            type Out = $ty;
            fn as_single(self) -> Self::Out {
                self
            }
        }
    };
}

as_single_self!(f32);
as_single_self!(Vec2);
as_single_self!(Vec3);
as_single_self!(Vec3A);
as_single_self!(Vec4);
as_single_self!(Quat);

impl AsSingle for f64 {
    type Out = f32;
    fn as_single(self) -> Self::Out {
        self as f32
    }
}

impl AsSingle for DVec2 {
    type Out = Vec2;
    fn as_single(self) -> Self::Out {
        self.as_vec2()
    }
}

impl AsSingle for DVec3 {
    type Out = Vec3;
    fn as_single(self) -> Self::Out {
        self.as_vec3()
    }
}

impl AsSingle for DVec4 {
    type Out = Vec4;
    fn as_single(self) -> Self::Out {
        self.as_vec4()
    }
}

impl AsSingle for DQuat {
    type Out = Quat;
    fn as_single(self) -> Self::Out {
        self.as_quat()
    }
}

#[cfg(feature = "f32")]
mod real {
    use super::{AsPrecise, AsSingle};
    use bevy::math::*;

    as_precise_self!(f32);
    as_precise_self!(Vec2);
    as_precise_self!(Vec3);
    as_precise_self!(Vec3A);
    as_precise_self!(Vec4);
    as_precise_self!(Quat);

    impl AsPrecise for f64 {
        type Out = f32;
        fn as_precise(self) -> Self::Out {
            self.as_single()
        }
    }

    impl AsPrecise for DVec2 {
        type Out = Vec2;
        fn as_precise(self) -> Self::Out {
            self.as_single()
        }
    }

    impl AsPrecise for DVec3 {
        type Out = Vec3;
        fn as_precise(self) -> Self::Out {
            self.as_single()
        }
    }

    impl AsPrecise for DVec4 {
        type Out = Vec4;
        fn as_precise(self) -> Self::Out {
            self.as_single()
        }
    }

    impl AsPrecise for DQuat {
        type Out = Quat;
        fn as_precise(self) -> Self::Out {
            self.as_single()
        }
    }
}

#[cfg(feature = "f64")]
mod real {
    use super::AsPrecise;
    use bevy::math::*;

    as_precise_self!(f64);
    as_precise_self!(DVec2);
    as_precise_self!(DVec3);
    as_precise_self!(DVec4);
    as_precise_self!(DQuat);

    impl AsPrecise for f32 {
        type Out = f64;
        fn as_precise(self) -> Self::Out {
            self as f64
        }
    }

    impl AsPrecise for Vec2 {
        type Out = DVec2;
        fn as_precise(self) -> Self::Out {
            self.as_dvec2()
        }
    }

    impl AsPrecise for Vec3 {
        type Out = DVec3;
        fn as_precise(self) -> Self::Out {
            self.as_dvec3()
        }
    }

    impl AsPrecise for Vec3A {
        type Out = DVec3;
        fn as_precise(self) -> Self::Out {
            self.as_dvec3()
        }
    }

    impl AsPrecise for Vec4 {
        type Out = DVec4;
        fn as_precise(self) -> Self::Out {
            self.as_dvec4()
        }
    }

    impl AsPrecise for Quat {
        type Out = DQuat;
        fn as_precise(self) -> Self::Out {
            self.as_dquat()
        }
    }
}
