//! This module defines the data element API, which enables NIfTI
//! volume API implementations to read, write and convert data
//! elements.
use std::io::Read;
use std::ops::{Mul, Add};
use std::mem::align_of;
use byteorder::ReadBytesExt;
use safe_transmute::guarded_transmute_pod_vec_permissive;
use error::Result;
use num_traits::cast::AsPrimitive;
use util::{Endianness, convert_bytes_to};

/// Interface for linear (affine) transformations to values. Multiple
/// implementations are needed because the original type `T` may not have
/// enough precision to obtain an appropriate outcome. For example,
/// transforming a `u8` is always done through `f32`, but an `f64` is instead
/// manipulated through its own type by first converting the slope and
/// intercept arguments to `f64`.
pub trait LinearTransform<T: 'static + Copy> {
    /// Linearly transform a value with the given slope and intercept.
    fn linear_transform(value: T, slope: f32, intercept: f32) -> T;

    /// Linearly transform a sequence of values with the given slope and intercept into
    /// a vector.
    fn linear_transform_many(value: &[T], slope: f32, intercept: f32) -> Vec<T> {
        value.iter()
            .map(|x| Self::linear_transform(*x, slope, intercept))
            .collect()
    }

    /// Linearly transform a sequence of values inline, with the given slope and intercept.
    fn linear_transform_many_inline(value: &mut [T], slope: f32, intercept: f32) {
        for v in value.iter_mut() {
            *v = Self::linear_transform(*v, slope, intercept);
        }
    }
}

/// A linear transformation in which the value is converted to `f32` for the
/// affine transformation, then converted back to the original type. Ideal for
/// small, low precision types such as `u8` and `i16`.
#[derive(Debug)]
pub struct LinearTransformViaF32;

impl<T> LinearTransform<T> for LinearTransformViaF32
where
    T: AsPrimitive<f32>,
    f32: AsPrimitive<T>,
{
    fn linear_transform(value: T, slope: f32, intercept: f32) -> T {
        if slope == 0. { return value }
        (value.as_() * slope + intercept).as_()
    }
}

/// A linear transformation in which the value and parameters are converted to
/// `f64` for the affine transformation, then converted to the original type.
/// Ideal for wide integer types such as `i64`.
#[derive(Debug)]
pub struct LinearTransformViaF64;

impl<T> LinearTransform<T> for LinearTransformViaF64
where
    T: 'static + Copy + AsPrimitive<f64>,
    f64: AsPrimitive<T>,
{
    fn linear_transform(value: T, slope: f32, intercept: f32) -> T {
        if slope == 0. { return value }
        let slope: f64 = slope.as_();
        let intercept: f64 = intercept.as_();
        (value.as_() * slope + intercept).as_()
    }
}

/// A linear transformation in which the slope and intercept parameters are
/// converted to the value's type for the affine transformation. Ideal
/// for high precision or complex number types.
#[derive(Debug)]
pub struct LinearTransformViaOriginal;

impl<T> LinearTransform<T> for LinearTransformViaOriginal
where
    T: 'static + DataElement + Mul<Output = T> + Add<Output = T> + Copy,
    f32: AsPrimitive<T>,
{
    fn linear_transform(value: T, slope: f32, intercept: f32) -> T {
        if slope == 0. { return value }
        let slope: T = slope.as_();
        let intercept: T = intercept.as_();
        value * slope + intercept
    }
}

/// Trait type for characterizing a NIfTI data element, implemented for
/// primitive numeric types which are used by the crate to represent voxel
/// values.
pub trait DataElement: 'static + Sized + Copy + AsPrimitive<u8> + AsPrimitive<f32> + AsPrimitive<f64>
{
    /// For defining how this element is linearly transformed to another.
    type Transform: LinearTransform<Self>;

    /// Read a single element from the given byte source.
    fn from_raw<R: Read>(src: R, endianness: Endianness) -> Result<Self>;

    /// Transform the given data vector into a vector of data elements.
    fn from_raw_vec(vec: Vec<u8>, endianness: Endianness) -> Result<Vec<Self>> {
        let mut cursor: &[u8] = &vec;
        let n = align_of::<Self>();
        (0..n).map(|_| Self::from_raw(&mut cursor, endianness)).collect()
    }
}

impl DataElement for u8 {
    type Transform = LinearTransformViaF32;
    fn from_raw_vec(vec: Vec<u8>, _: Endianness) -> Result<Vec<Self>> {
        Ok(vec)
    }
    fn from_raw<R: Read>(mut src: R, _: Endianness) -> Result<Self> {
        src.read_u8().map_err(From::from)
    }
}
impl DataElement for i8 {
    type Transform = LinearTransformViaF32;
    fn from_raw_vec(vec: Vec<u8>, _: Endianness) -> Result<Vec<Self>> {
        Ok(guarded_transmute_pod_vec_permissive(vec))
    }
    fn from_raw<R: Read>(mut src: R, _: Endianness) -> Result<Self> {
        src.read_i8().map_err(From::from)
    }
}
impl DataElement for u16 {
    type Transform = LinearTransformViaF32;
    fn from_raw_vec(vec: Vec<u8>, e: Endianness) -> Result<Vec<Self>> {
        Ok(convert_bytes_to(vec, e))
    }
    fn from_raw<R: Read>(src: R, e: Endianness) -> Result<Self> {
        e.read_u16(src).map_err(From::from)
    }
}
impl DataElement for i16 {
    type Transform = LinearTransformViaF32;
    fn from_raw_vec(vec: Vec<u8>, e: Endianness) -> Result<Vec<Self>> {
        Ok(convert_bytes_to(vec, e))
    }
    fn from_raw<R: Read>(src: R, e: Endianness) -> Result<Self> {
        e.read_i16(src).map_err(From::from)
    }
}
impl DataElement for u32 {
    type Transform = LinearTransformViaF32;
    fn from_raw_vec(vec: Vec<u8>, e: Endianness) -> Result<Vec<Self>> {
        Ok(convert_bytes_to(vec, e))
    }
    fn from_raw<R: Read>(src: R, e: Endianness) -> Result<Self> {
        e.read_u32(src).map_err(From::from)
    }
}
impl DataElement for i32 {
    type Transform = LinearTransformViaF32;
    fn from_raw_vec(vec: Vec<u8>, e: Endianness) -> Result<Vec<Self>> {
        Ok(convert_bytes_to(vec, e))
    }
    fn from_raw<R: Read>(src: R, e: Endianness) -> Result<Self> {
        e.read_i32(src).map_err(From::from)
    }
}
impl DataElement for u64 {
    type Transform = LinearTransformViaF64;
    fn from_raw_vec(vec: Vec<u8>, e: Endianness) -> Result<Vec<Self>> {
        Ok(convert_bytes_to(vec, e))
    }
    fn from_raw<R: Read>(src: R, e: Endianness) -> Result<Self> {
        e.read_u64(src).map_err(From::from)
    }
}
impl DataElement for i64 {
    type Transform = LinearTransformViaF64;
    fn from_raw_vec(vec: Vec<u8>, e: Endianness) -> Result<Vec<Self>> {
        Ok(convert_bytes_to(vec, e))
    }
    fn from_raw<R: Read>(src: R, e: Endianness) -> Result<Self> {
        e.read_i64(src).map_err(From::from)
    }
}
impl DataElement for f32 {
    type Transform = LinearTransformViaOriginal;
    fn from_raw_vec(vec: Vec<u8>, e: Endianness) -> Result<Vec<Self>> {
        Ok(convert_bytes_to(vec, e))
    }
    fn from_raw<R: Read>(src: R, e: Endianness) -> Result<Self> {
        e.read_f32(src).map_err(From::from)
    }
}
impl DataElement for f64 {
    type Transform = LinearTransformViaOriginal;
    fn from_raw_vec(vec: Vec<u8>, e: Endianness) -> Result<Vec<Self>> {
        Ok(convert_bytes_to(vec, e))
    }
    fn from_raw<R: Read>(src: R, e: Endianness) -> Result<Self> {
        e.read_f64(src).map_err(From::from)
    }
}
