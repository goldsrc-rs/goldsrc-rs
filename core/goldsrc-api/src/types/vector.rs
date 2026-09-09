//! 3D Vector mathematics for GoldSrc positions, velocities, and angles.
//!
//! Provides binary FFI layout (`#[repr(C)]`) matching engine `vec3_t`, complete
//! arithmetic operators (`+`, `-`, `*`, `/`, `-v`), spatial queries (distance, length),
//! linear algebra (dot, cross, normalize, reflect), and angle conversions based on
//! AMX Mod X `vector.inc` and `xs.inc`.

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// 3D Vector type for GoldSrc positions, velocities, and Euler angles.
///
/// Guaranteed `#[repr(C)]` layout matching engine `vec3_t` (`[f32; 3]`).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector3 {
    /// X component (forward / pitch).
    pub x: f32,
    /// Y component (right / yaw).
    pub y: f32,
    /// Z component (up / roll).
    pub z: f32,
}

impl Vector3 {
    /// Zero vector `(0.0, 0.0, 0.0)`.
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    /// Unit vector along the X axis `(1.0, 0.0, 0.0)`.
    pub const UNIT_X: Self = Self::new(1.0, 0.0, 0.0);

    /// Unit vector along the Y axis `(0.0, 1.0, 0.0)`.
    pub const UNIT_Y: Self = Self::new(0.0, 1.0, 0.0);

    /// Unit vector along the Z axis `(0.0, 0.0, 1.0)`.
    pub const UNIT_Z: Self = Self::new(0.0, 0.0, 1.0);

    /// Creates a new 3D vector.
    #[inline(always)]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a 3D vector on the horizontal plane (`z = 0.0`).
    #[inline(always)]
    pub const fn from_2d(x: f32, y: f32) -> Self {
        Self { x, y, z: 0.0 }
    }

    /// Converts integer coordinates to floating point vector (`IVecFVec`).
    #[inline(always)]
    pub fn from_i32(ivec: [i32; 3]) -> Self {
        Self {
            x: ivec[0] as f32,
            y: ivec[1] as f32,
            z: ivec[2] as f32,
        }
    }

    /// Converts floating point vector to rounded integer array (`FVecIVec`).
    #[inline(always)]
    pub fn to_i32(&self) -> [i32; 3] {
        [
            self.x.round() as i32,
            self.y.round() as i32,
            self.z.round() as i32,
        ]
    }

    /// Sets the vector components to specified values (`xs_vec_set`).
    #[inline(always)]
    pub fn set(&mut self, x: f32, y: f32, z: f32) {
        self.x = x;
        self.y = y;
        self.z = z;
    }

    /// Copies components from another vector (`xs_vec_copy`).
    #[inline(always)]
    pub fn copy_from(&mut self, other: Self) {
        *self = other;
    }

    /// Linear interpolation between `self` and `other` with factor `t` (`0.0..=1.0`).
    #[inline(always)]
    pub fn lerp(&self, other: Self, t: f32) -> Self {
        *self + (other - *self) * t
    }

    /// Returns the 2D components `[x, y]` ignoring `z` (`xs_vec_make2d`).
    #[inline(always)]
    pub const fn make_2d(&self) -> [f32; 2] {
        [self.x, self.y]
    }

    /// Returns `true` if all components are zero.
    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }

    /// Checks if two vectors are approximately equal within `epsilon` (`xs_vec_nearlyequal`).
    #[inline]
    pub fn nearly_equal(&self, other: Self, epsilon: f32) -> bool {
        (self.x - other.x).abs() <= epsilon
            && (self.y - other.y).abs() <= epsilon
            && (self.z - other.z).abs() <= epsilon
    }

    /// Computes the dot product of two vectors (`xs_vec_dot`).
    #[inline(always)]
    pub fn dot(&self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Computes the cross product of two vectors (`xs_vec_cross`).
    #[inline(always)]
    pub fn cross(&self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Computes the Euclidean length of the vector (`xs_vec_len` / `vector_length`).
    #[inline(always)]
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Computes the squared length of the vector (faster than `length`).
    #[inline(always)]
    pub fn length_squared(&self) -> f32 {
        self.dot(*self)
    }

    /// Computes the horizontal 2D length ignoring `z` (`xs_vec_len_2d`).
    #[inline(always)]
    pub fn length_2d(&self) -> f32 {
        self.length_2d_squared().sqrt()
    }

    /// Computes the squared horizontal 2D length ignoring `z`.
    #[inline(always)]
    pub fn length_2d_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// Computes the distance between two points (`xs_vec_distance` / `get_distance_f`).
    #[inline(always)]
    pub fn distance(&self, other: Self) -> f32 {
        (*self - other).length()
    }

    /// Computes the squared distance between two points (`xs_vec_sqdistance`).
    #[inline(always)]
    pub fn distance_squared(&self, other: Self) -> f32 {
        (*self - other).length_squared()
    }

    /// Computes the 2D horizontal distance ignoring `z` (`xs_vec_distance_2d`).
    #[inline(always)]
    pub fn distance_2d(&self, other: Self) -> f32 {
        (*self - other).length_2d()
    }

    /// Computes the squared 2D horizontal distance ignoring `z` (`xs_vec_sqdistance_2d`).
    #[inline(always)]
    pub fn distance_2d_squared(&self, other: Self) -> f32 {
        (*self - other).length_2d_squared()
    }

    /// Returns a normalized unit vector with length 1.0 (`xs_vec_normalize`).
    /// Returns `Vector3::ZERO` if the vector length is zero.
    #[inline]
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 1e-6 { *self / len } else { Self::ZERO }
    }

    /// Adds `other` scaled by `scalar` to `self` (`xs_vec_add_scaled`).
    #[inline(always)]
    pub fn add_scaled(&self, other: Self, scalar: f32) -> Self {
        *self + other * scalar
    }

    /// Subtracts `other` scaled by `scalar` from `self` (`xs_vec_sub_scaled`).
    #[inline(always)]
    pub fn sub_scaled(&self, other: Self, scalar: f32) -> Self {
        *self - other * scalar
    }

    /// Computes the angle in degrees between two vectors (`xs_vec_angle`).
    #[inline]
    pub fn angle_between(&self, other: Self) -> f32 {
        let denom = (self.length_squared() * other.length_squared()).sqrt();
        if denom < 1e-6 {
            return 0.0;
        }
        let cos_theta = (self.dot(other) / denom).clamp(-1.0, 1.0);
        cos_theta.acos().to_degrees()
    }

    /// Reflects this vector about a surface normal (`xs_vec_reflect`).
    #[inline]
    pub fn reflect(&self, normal: Self) -> Self {
        let n = normal.normalize();
        *self - n * (2.0 * self.dot(n))
    }

    /// Converts Euler rotation angles (Pitch, Yaw, Roll in degrees) into directional
    /// forward, right, and up unit vectors (`AngleVectors` / `angle_vector`).
    pub fn angle_vectors(&self) -> (Vector3, Vector3, Vector3) {
        let pitch = self.x.to_radians();
        let yaw = self.y.to_radians();
        let roll = self.z.to_radians();

        let (sin_p, cos_p) = pitch.sin_cos();
        let (sin_y, cos_y) = yaw.sin_cos();
        let (sin_r, cos_r) = roll.sin_cos();

        let forward = Vector3::new(cos_p * cos_y, cos_p * sin_y, -sin_p);
        let right = Vector3::new(
            -sin_r * sin_p * cos_y + cos_r * sin_y,
            -sin_r * sin_p * sin_y - cos_r * cos_y,
            -sin_r * cos_p,
        );
        let up = Vector3::new(
            cos_r * sin_p * cos_y + sin_r * sin_y,
            cos_r * sin_p * sin_y - sin_r * cos_y,
            cos_r * cos_p,
        );

        (forward, right, up)
    }

    /// Returns the forward directional unit vector for these Euler angles.
    #[inline]
    pub fn forward_vector(&self) -> Vector3 {
        let pitch = self.x.to_radians();
        let yaw = self.y.to_radians();
        let (sin_p, cos_p) = pitch.sin_cos();
        let (sin_y, cos_y) = yaw.sin_cos();
        Vector3::new(cos_p * cos_y, cos_p * sin_y, -sin_p)
    }

    /// Returns the right directional unit vector for these Euler angles.
    #[inline]
    pub fn right_vector(&self) -> Vector3 {
        self.angle_vectors().1
    }

    /// Returns the up directional unit vector for these Euler angles.
    #[inline]
    pub fn up_vector(&self) -> Vector3 {
        self.angle_vectors().2
    }

    /// Calculates velocity vector in the direction of given Euler angles (`velocity_by_aim`).
    #[inline(always)]
    pub fn velocity_by_aim(angles: Vector3, speed: f32) -> Vector3 {
        angles.forward_vector() * speed
    }

    /// Converts a directional vector into GoldSrc Euler angles `(pitch, yaw, 0)` (`VectorAngles` / `vector_to_angle`).
    pub fn to_angles(&self) -> Vector3 {
        if self.x == 0.0 && self.y == 0.0 {
            let pitch = if self.z > 0.0 { 270.0 } else { 90.0 };
            return Vector3::new(pitch, 0.0, 0.0);
        }

        let yaw = self.y.atan2(self.x).to_degrees();
        let yaw = if yaw < 0.0 { yaw + 360.0 } else { yaw };

        let forward_2d = self.length_2d();
        let pitch = (-self.z).atan2(forward_2d).to_degrees();
        let pitch = if pitch < 0.0 { pitch + 360.0 } else { pitch };

        Vector3::new(pitch, yaw, 0.0)
    }
}

// --- Operator implementations ---

impl Add for Vector3 {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Vector3 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vector3 {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl SubAssign for Vector3 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z += rhs.z;
    }
}

impl Mul<f32> for Vector3 {
    type Output = Self;
    #[inline(always)]
    fn mul(self, scalar: f32) -> Self::Output {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl MulAssign<f32> for Vector3 {
    #[inline(always)]
    fn mul_assign(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }
}

impl Mul<Vector3> for f32 {
    type Output = Vector3;
    #[inline(always)]
    fn mul(self, vec: Vector3) -> Self::Output {
        vec * self
    }
}

impl Div<f32> for Vector3 {
    type Output = Self;
    #[inline(always)]
    fn div(self, scalar: f32) -> Self::Output {
        let inv = 1.0 / scalar;
        Self {
            x: self.x * inv,
            y: self.y * inv,
            z: self.z * inv,
        }
    }
}

impl DivAssign<f32> for Vector3 {
    #[inline(always)]
    fn div_assign(&mut self, scalar: f32) {
        let inv = 1.0 / scalar;
        self.x *= inv;
        self.y *= inv;
        self.z *= inv;
    }
}

impl Neg for Vector3 {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl From<[f32; 3]> for Vector3 {
    #[inline(always)]
    fn from(arr: [f32; 3]) -> Self {
        Self {
            x: arr[0],
            y: arr[1],
            z: arr[2],
        }
    }
}

impl From<Vector3> for [f32; 3] {
    #[inline(always)]
    fn from(v: Vector3) -> Self {
        [v.x, v.y, v.z]
    }
}

impl From<(f32, f32, f32)> for Vector3 {
    #[inline(always)]
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self { x, y, z }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_basic_arithmetic() {
        let a = Vector3::new(1.0, 2.0, 3.0);
        let b = Vector3::new(4.0, 5.0, 6.0);

        assert_eq!(a + b, Vector3::new(5.0, 7.0, 9.0));
        assert_eq!(b - a, Vector3::new(3.0, 3.0, 3.0));
        assert_eq!(a * 2.0, Vector3::new(2.0, 4.0, 6.0));
        assert_eq!(2.0 * a, Vector3::new(2.0, 4.0, 6.0));
        assert_eq!(b / 2.0, Vector3::new(2.0, 2.5, 3.0));
        assert_eq!(-a, Vector3::new(-1.0, -2.0, -3.0));
    }

    #[test]
    fn test_vector_spatial_geometry() {
        let a = Vector3::new(3.0, 4.0, 0.0);
        assert_eq!(a.length(), 5.0);
        assert_eq!(a.length_squared(), 25.0);

        let norm = a.normalize();
        assert!((norm.length() - 1.0).abs() < 1e-6);
        assert_eq!(norm, Vector3::new(0.6, 0.8, 0.0));

        let b = Vector3::new(0.0, 0.0, 0.0);
        assert_eq!(a.distance(b), 5.0);
        assert_eq!(a.distance_squared(b), 25.0);
    }

    #[test]
    fn test_vector_dot_and_cross() {
        let x = Vector3::UNIT_X;
        let y = Vector3::UNIT_Y;
        assert_eq!(x.dot(y), 0.0);
        assert_eq!(x.cross(y), Vector3::UNIT_Z);
        assert_eq!(y.cross(x), -Vector3::UNIT_Z);
    }

    #[test]
    fn test_angle_vectors_roundtrip() {
        let angles = Vector3::new(0.0, 90.0, 0.0); // Facing positive Y (North)
        let fwd = angles.forward_vector();
        assert!(fwd.nearly_equal(Vector3::new(0.0, 1.0, 0.0), 1e-4));

        let back_to_angles = fwd.to_angles();
        assert!((back_to_angles.y - 90.0).abs() < 1e-3);
    }
}
