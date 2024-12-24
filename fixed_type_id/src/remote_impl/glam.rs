use glam::*;

use crate::prelude::*;

fixed_type_id! {
    // we remove the `f32` prefix for the type name
    glam::Vec2;
    glam::Vec3;
    glam::Vec3A;
    glam::Vec4;
    glam::Mat2;
    glam::Mat3;
    glam::Mat3A;
    glam::Mat4;
    glam::Quat;
    glam::Affine2;
    glam::Affine3A;
    // we remove the `f64` prefix for the type name
    glam::DVec2;
    glam::DVec3;
    glam::DVec4;
    glam::DMat2;
    glam::DMat3;
    glam::DMat4;
    glam::DQuat;
    glam::DAffine2;
    glam::DAffine3;
    // we remove the `i8` prefix for the type name
    glam::I8Vec2;
    glam::I8Vec3;
    glam::I8Vec4;
    // we remove the `u8` prefix for the type name
    glam::U8Vec2;
    glam::U8Vec3;
    glam::U8Vec4;
    // we remove the `i16` prefix for the type name
    glam::I16Vec2;
    glam::I16Vec3;
    glam::I16Vec4;
    // we remove the `u16` prefix for the type name
    glam::U16Vec2;
    glam::U16Vec3;
    glam::U16Vec4;
    // we remove the `i32` prefix for the type name
    glam::IVec2;
    glam::IVec3;
    glam::IVec4;
    // we remove the `u32` prefix for the type name
    glam::UVec2;
    glam::UVec3;
    glam::UVec4;
    // we remove the `i64` prefix for the type name
    glam::I64Vec2;
    glam::I64Vec3;
    glam::I64Vec4;
    // we remove the `u64` prefix for the type name
    glam::U64Vec2;
    glam::U64Vec3;
    glam::U64Vec4;
    // we remove the `bool` prefix for the type name
    glam::BVec2;
    glam::BVec3;
    glam::BVec4;

    glam::EulerRot;
}
