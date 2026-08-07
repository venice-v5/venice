use std::{cell::Cell, fmt::Write};

use argparse::{ArgType, Args, error_msg};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    except::{ZERO_DIVISION_ERROR_TYPE, raise_msg, type_error},
    init::token,
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    ops::{BinaryOpCode, UnaryOpCode},
    print::{Print, PrintKind},
    qstr::Qstr,
};
use mint::{EulerAngles, IntraZYX};
use vexide_devices::math::Angle;

use crate::{
    modvenice::{Exception, units::rotation::RotationUnit},
    obj::alloc_obj,
};

/// A mutable three-component floating-point vector used by Venice device APIs.
///
/// `x`, `y`, and `z` are mutable `float` attributes. Their physical units depend on the API that
/// produced the vector; for example, inertial APIs use these components for acceleration or
/// angular velocity. Assigning an `int` or `float` updates a component, and deleting a component
/// resets it to `0.0`.
///
/// Vectors support unary `+` and `-`, component-wise `+` and `-`, scalar multiplication and true
/// division, component-wise exponentiation by a scalar, and exact component-wise equality.
/// In-place operators return a new vector rather than mutating the original. Division by zero
/// raises `ZeroDivisionError`.
#[class(qstr!(Vec3))]
#[repr(C)]
pub struct Vec3 {
    base: ObjBase,
    x: Cell<f32>,
    y: Cell<f32>,
    z: Cell<f32>,
}

/// A mutable quaternion represented by four floating-point components.
///
/// `w` is the mutable scalar component, and `x`, `y`, and `z` are the mutable imaginary
/// components. Assigning an `int` or `float` updates a component. Deleting any component resets it
/// to `0.0`, including `w`. Quaternions support exact component-wise equality and have a readable
/// `Quaternion(w=..., x=..., y=..., z=...)` representation.
#[class(qstr!(Quaternion))]
#[repr(C)]
pub struct Quaternion {
    base: ObjBase,
    // i
    x: Cell<f32>,
    // j
    y: Cell<f32>,
    // k
    z: Cell<f32>,
    // real
    w: Cell<f32>,
}

/// Mutable intrinsic Z-Y-X Euler angles.
///
/// `yaw`, `pitch`, and `roll` are mutable `float` attributes describing rotations about the Z, Y,
/// and X axes, respectively. Their angular unit is determined by the producer, such as the `unit`
/// passed to `InertialSensor.get_euler` or `GpsSensor.get_euler`; directly constructed values have
/// no stored unit. Assigning an `int` or `float` updates an angle, and deleting one resets it to
/// `0.0`. Instances support exact component-wise equality and a readable representation.
#[class(qstr!(EulerZYX))]
#[repr(C)]
pub struct EulerZYX {
    base: ObjBase,
    yaw: Cell<f32>,
    pitch: Cell<f32>,
    roll: Cell<f32>,
}

/// A mutable point in two-dimensional Cartesian coordinates.
///
/// `x` and `y` are mutable `float` attributes. Their physical unit depends on the consuming API;
/// GPS positions and offsets use metres. Assigning an `int` or `float` updates a coordinate, and
/// deleting one resets it to `0.0`. Points support exact coordinate-wise equality and have a
/// readable `Point2(x=..., y=...)` representation.
#[class(qstr!(Point2))]
#[derive(Clone)]
#[repr(C)]
pub struct Point2 {
    base: ObjBase,
    x: Cell<f32>,
    y: Cell<f32>,
}

#[class_methods]
impl Vec3 {
    pub fn new(v: vexide_devices::math::Vector3<f64>) -> Self {
        Self {
            base: Self::OBJ_TYPE.into(),
            x: Cell::new(v.x as f32),
            y: Cell::new(v.y as f32),
            z: Cell::new(v.z as f32),
        }
    }

    /// Creates a vector with components `x`, `y`, and `z`, each defaulting to `0.0`.
    ///
    /// All three arguments are positional-only and accept either `int` or `float` values.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If a component is not numeric, a keyword argument is supplied, or more than
    ///   three positional arguments are given.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// vector = Vec3(1.0, 2.0, 3.0)
    /// ```
    #[make_new]
    #[stub(sig = "(self, x: float = 0.0, y: float = 0.0, z: float = 0.0) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(0, 3).assert_nkw(0, 0);

        let x = reader.next_positional_or(0.0)?;
        let y = reader.next_positional_or(0.0)?;
        let z = reader.next_positional_or(0.0)?;

        Ok(Self {
            base: ty.into(),
            x: x.into(),
            y: y.into(),
            z: z.into(),
        })
    }

    /// Loads, stores, or deletes the mutable `x`, `y`, and `z` attributes.
    ///
    /// Assigning a non-numeric value raises `TypeError`; deleting an attribute resets that component to
    /// `0.0`.
    #[attr]
    #[stub(attrs = ["x: float", "y: float", "z: float"])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let coord = match attr.as_str() {
            "x" => &self.x,
            "y" => &self.y,
            "z" => &self.z,
            _ => return,
        };

        handle_op(op, coord);
    }

    /// Implements unary plus and negation.
    ///
    /// Unary plus returns the same object, while negation returns a new `Vec3` with every component
    /// negated.
    #[unary_op]
    fn unary_op(op: UnaryOpCode, obj: &Obj) -> Obj {
        match op {
            UnaryOpCode::Positive => *obj,
            UnaryOpCode::Negative => {
                let vec3 = obj.as_obj::<Self>();
                alloc_obj(Self {
                    base: Self::OBJ_TYPE.into(),
                    x: Cell::new(-vec3.x.get()),
                    y: Cell::new(-vec3.y.get()),
                    z: Cell::new(-vec3.z.get()),
                })
            }
            _ => Obj::NULL,
        }
    }

    fn eq(lhs: &Self, rhs: &Self) -> bool {
        lhs.x.get() == rhs.x.get() && lhs.y.get() == rhs.y.get() && lhs.z.get() == rhs.z.get()
    }

    /// Implements exact equality with another `Vec3`, vector addition and subtraction, and scalar
    /// multiplication, division, and exponentiation.
    ///
    /// Arithmetic returns a new vector. The scalar must be convertible to `float`; unsupported operand
    /// combinations follow Python's normal binary-operation fallback. Division by `0` or `0.0`
    /// raises `ZeroDivisionError`.
    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(Self::eq(lhs, rhs.try_as_obj::<Self>().unwrap())),
            BinaryOpCode::Add | BinaryOpCode::InplaceAdd => {
                let rhs = match rhs.try_as_obj::<Self>() {
                    Some(r) => r,
                    _ => return Obj::NULL,
                };

                Obj::from(Self {
                    base: Self::OBJ_TYPE.into(),
                    x: Cell::new(lhs.x.get() + rhs.x.get()),
                    y: Cell::new(lhs.y.get() + rhs.y.get()),
                    z: Cell::new(lhs.z.get() + rhs.z.get()),
                })
            }
            BinaryOpCode::Subtract | BinaryOpCode::InplaceSubtract => {
                let rhs = match rhs.try_as_obj::<Self>() {
                    Some(r) => r,
                    _ => return Obj::NULL,
                };

                Obj::from(Self {
                    base: Self::OBJ_TYPE.into(),
                    x: Cell::new(lhs.x.get() - rhs.x.get()),
                    y: Cell::new(lhs.y.get() - rhs.y.get()),
                    z: Cell::new(lhs.z.get() - rhs.z.get()),
                })
            }
            BinaryOpCode::Multiply
            | BinaryOpCode::InplaceMultiply
            | BinaryOpCode::ReverseMultiply => {
                let rhs = match rhs
                    .try_to_float()
                    .or_else(|| rhs.try_to_int().map(|i| i as f32))
                {
                    Some(r) => r,
                    None => return Obj::NULL,
                };

                Obj::from(Self {
                    base: Self::OBJ_TYPE.into(),
                    x: Cell::new(lhs.x.get() * rhs),
                    y: Cell::new(lhs.y.get() * rhs),
                    z: Cell::new(lhs.z.get() * rhs),
                })
            }
            BinaryOpCode::TrueDivide | BinaryOpCode::InplaceTrueDivide => {
                let rhs = match rhs
                    .try_to_float()
                    .or_else(|| rhs.try_to_int().map(|i| i as f32))
                {
                    Some(r) => r,
                    None => return Obj::NULL,
                };

                if rhs == 0.0 {
                    raise_msg(token(), ZERO_DIVISION_ERROR_TYPE, c"divison by zero")
                }

                Obj::from(Self {
                    base: Self::OBJ_TYPE.into(),
                    x: Cell::new(lhs.x.get() / rhs),
                    y: Cell::new(lhs.y.get() / rhs),
                    z: Cell::new(lhs.z.get() / rhs),
                })
            }
            BinaryOpCode::Power | BinaryOpCode::InplacePower => {
                let rhs = match rhs
                    .try_to_float()
                    .or_else(|| rhs.try_to_int().map(|i| i as f32))
                {
                    Some(r) => r,
                    None => return Obj::NULL,
                };

                Obj::from(Self {
                    base: Self::OBJ_TYPE.into(),
                    x: Cell::new(lhs.x.get().powf(rhs)),
                    y: Cell::new(lhs.y.get().powf(rhs)),
                    z: Cell::new(lhs.z.get().powf(rhs)),
                })
            }
            _ => Obj::NULL,
        }
    }

    /// Formats the vector as `Vec3(x=..., y=..., z=...)`.
    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        let _ = write!(
            print,
            "Vec3(x={}, y={}, z={})",
            self.x.get(),
            self.y.get(),
            self.z.get()
        );
    }
}

#[class_methods]
impl Quaternion {
    pub fn new(quat: vexide_devices::math::Quaternion<f64>) -> Self {
        Self {
            base: Self::OBJ_TYPE.into(),
            x: Cell::new(quat.v.x as f32),
            y: Cell::new(quat.v.y as f32),
            z: Cell::new(quat.v.z as f32),
            w: Cell::new(quat.s as f32),
        }
    }

    /// Creates a quaternion with scalar component `w` and imaginary components `x`, `y`, and `z`.
    ///
    /// Every component defaults to `0.0`. All arguments are positional-only, accept either `int`
    /// or `float`, and are stored without normalization.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If a component is not numeric, a keyword argument is supplied, or more than
    ///   four positional arguments are given.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// quaternion = Quaternion(1.0, 0.0, 0.0, 0.0)
    /// ```
    #[make_new]
    #[stub(sig = "(self, w: float = 1.0, x: float = 0.0, y: float = 0.0, z: float = 0.0) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(0, 4).assert_nkw(0, 0);

        let w = reader.next_positional_or(1.0)?;
        let x = reader.next_positional_or(0.0)?;
        let y = reader.next_positional_or(0.0)?;
        let z = reader.next_positional_or(0.0)?;

        Ok(Self {
            base: ty.into(),
            w: w.into(),
            x: x.into(),
            y: y.into(),
            z: z.into(),
        })
    }

    /// Loads, stores, or deletes the mutable `w`, `x`, `y`, and `z` attributes.
    ///
    /// Assigning a non-numeric value raises `TypeError`; deleting an attribute resets that component to
    /// `0.0`.
    #[attr]
    #[stub(attrs = ["w: float", "x: float", "y: float", "z: float"])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let val = match attr.as_str() {
            "x" => &self.x,
            "y" => &self.y,
            "z" => &self.z,
            "w" => &self.w,
            _ => return,
        };

        handle_op(op, val);
    }

    fn eq(lhs: &Self, rhs: &Self) -> bool {
        lhs.w.get() == rhs.w.get()
            && lhs.x.get() == rhs.x.get()
            && lhs.y.get() == rhs.y.get()
            && lhs.z.get() == rhs.z.get()
    }

    /// Implements exact component-wise equality with another `Quaternion`.
    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(Self::eq(lhs, rhs.as_obj())),
            _ => Obj::NULL,
        }
    }

    /// Formats the quaternion as `Quaternion(w=..., x=..., y=..., z=...)`.
    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        let _ = write!(
            print,
            "Quaternion(w={}, x={}, y={}, z={})",
            self.w.get(),
            self.x.get(),
            self.y.get(),
            self.z.get()
        );
    }
}

#[class_methods]
impl EulerZYX {
    pub fn new(e: EulerAngles<Angle, IntraZYX>, unit: RotationUnit) -> Self {
        Self {
            base: ObjBase::new(Self::OBJ_TYPE),
            yaw: Cell::new(unit.angle_to_float(e.b)),
            pitch: Cell::new(unit.angle_to_float(e.a)),
            roll: Cell::new(unit.angle_to_float(e.c)),
        }
    }

    /// Creates intrinsic Z-Y-X Euler angles `yaw`, `pitch`, and `roll`.
    ///
    /// Every angle defaults to `0.0`. The constructor stores numeric values without attaching or
    /// converting an angular unit. All arguments are positional-only and accept either `int` or
    /// `float` values.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If an angle is not numeric, a keyword argument is supplied, or more than
    ///   three positional arguments are given.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// angles = EulerZYX(1.0, 0.0, 0.0)
    /// ```
    #[make_new]
    #[stub(sig = "(self, yaw: float = 0.0, pitch: float = 0.0, roll: float = 0.0) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(0, 3).assert_nkw(0, 0);

        let yaw = reader.next_positional_or(0.0)?;
        let pitch = reader.next_positional_or(0.0)?;
        let roll = reader.next_positional_or(0.0)?;

        Ok(Self {
            base: ty.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            roll: roll.into(),
        })
    }

    /// Loads, stores, or deletes the mutable `yaw`, `pitch`, and `roll` attributes.
    ///
    /// Assigning a non-numeric value raises `TypeError`; deleting an attribute resets that angle
    /// to `0.0`. The angular unit is determined by the API that produced or consumes the object.
    #[attr]
    #[stub(attrs = ["yaw: float", "pitch: float", "roll: float"])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let val = match attr.as_str() {
            "yaw" => &self.yaw,
            "pitch" => &self.pitch,
            "roll" => &self.roll,
            _ => return,
        };

        handle_op(op, val);
    }

    fn eq(lhs: &Self, rhs: &Self) -> bool {
        lhs.yaw.get() == rhs.yaw.get()
            && lhs.pitch.get() == rhs.pitch.get()
            && lhs.roll.get() == rhs.roll.get()
    }

    /// Implements exact component-wise equality with another `EulerZYX`.
    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(Self::eq(lhs, rhs.as_obj())),
            _ => Obj::NULL,
        }
    }

    /// Formats the angles as `EulerZYX(yaw=..., pitch=..., roll=...)`.
    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        let _ = write!(
            print,
            "EulerZYX(yaw={}, pitch={}, roll={})",
            self.yaw.get(),
            self.pitch.get(),
            self.roll.get(),
        );
    }
}

#[class_methods]
impl Point2 {
    /// Creates a point with coordinates `x` and `y`, each defaulting to `0.0`.
    ///
    /// Both arguments are positional-only and accept either `int` or `float` values. The point
    /// stores no unit; the API that consumes it determines the coordinate scale.
    ///
    /// # Raises
    ///
    /// - `TypeError`: If a coordinate is not numeric, a keyword argument is supplied, or more than
    ///   two positional arguments are given.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// point = Point2(10.0, 20.0)
    /// ```
    #[make_new]
    #[stub(sig = "(self, x: float = 0.0, y: float = 0.0) -> None")]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(0, 2).assert_nkw(0, 0);

        let x = reader.next_positional_or(0.0)?;
        let y = reader.next_positional_or(0.0)?;

        Ok(Self {
            base: ty.into(),
            x: x.into(),
            y: y.into(),
        })
    }

    /// Loads, stores, or deletes the mutable `x` and `y` attributes.
    ///
    /// Assigning a non-numeric value raises `TypeError`; deleting an attribute resets that coordinate to
    /// `0.0`.
    #[attr]
    #[stub(attrs = ["x: float", "y: float"])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let coord = match attr.as_str() {
            "x" => &self.x,
            "y" => &self.y,
            _ => return,
        };

        handle_op(op, coord);
    }

    pub fn from_vexide_point2(point2: vexide_devices::math::Point2<f64>) -> Self {
        Self {
            base: Self::OBJ_TYPE.into(),
            x: Cell::new(point2.x as f32),
            y: Cell::new(point2.y as f32),
        }
    }

    pub fn as_vexide_point2(&self) -> vexide_devices::math::Point2<f64> {
        vexide_devices::math::Point2 {
            x: self.x.get() as f64,
            y: self.y.get() as f64,
        }
    }

    fn eq(lhs: &Self, rhs: &Self) -> bool {
        lhs.x.get() == rhs.x.get() && lhs.y.get() == rhs.y.get()
    }

    /// Implements exact coordinate-wise equality with another `Point2`.
    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(Self::eq(lhs, rhs.as_obj())),
            _ => Obj::NULL,
        }
    }

    /// Formats the point as `Point2(x=..., y=...)`.
    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        let _ = write!(print, "Point2(x={}, y={})", self.x.get(), self.y.get());
    }
}

impl From<vexide_devices::math::Point2<f64>> for Point2 {
    fn from(value: vexide_devices::math::Point2<f64>) -> Self {
        Self::from_vexide_point2(value)
    }
}

pub fn handle_op(op: AttrOp, val: &Cell<f32>) {
    match op {
        AttrOp::Load { result } => result.return_value(Obj::from_float(val.get())),
        AttrOp::Store { src, result } => {
            if let Some(f) = src.try_to_int().map(|i| i as f32).or(src.try_to_float()) {
                val.set(f);
                result.success();
            } else {
                type_error(error_msg!("expected f32, found <{}>", ArgType::of(&src)))
                    .raise(token());
            }
        }
        AttrOp::Delete { result } => {
            val.set(0.0);
            // "sucess" bruh
            result.sucess();
        }
    }
}
