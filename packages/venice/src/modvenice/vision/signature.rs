use std::fmt::Write;

use argparse::Args;
use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    ops::BinaryOpCode,
    print::{Print, PrintKind},
    qstr::Qstr,
};
use vexide_devices::smart::vision::VisionSignature;

use crate::modvenice::{Exception, read_only_attr::read_only_attr};

/// A vision detection color signature.
///
/// This class is root-importable. Vision signatures contain information used by the Vision Sensor to detect objects of a certain
/// color. These signatures are typically generated through VEX's Vision Utility tool rather than
/// written by hand.
///
/// # Format & Detection Overview
///
/// Vision signatures operate in a version of the Y'UV color space, specifically using the "U" and
/// "V" chroma components for edge detection purposes. The read-only `u_min`, `u_max`, and `u_mean`
/// attributes place three threshold values on the U chroma values detected by the sensor. The
/// read-only `v_min`, `v_max`, and `v_mean` attributes do the same for the V component. These values
/// are then transformed to a 3D lookup table to detect actual colors.
///
/// The read-only `range` attribute works as a scale factor or threshold for how lenient edge detection
/// should be. It ranges from 0-11 in Vision Utility. Higher values increase the range of brightness
/// that the sensor considers part of the signature, so lighter and darker shades are detected more
/// often. The read-only `flags` attribute is the signature's flags and is initialized to 0.
///
/// Signatures can additionally be grouped together into `VisionCode` objects, which narrow the filter
/// for object detection by requiring two colors. Signatures compare equal when all thresholds,
/// `range`, and `flags` match, return `False` when compared with another type, and have a readable
/// representation containing those eight values.
#[class(qstr!(VisionSignature))]
#[repr(C)]
pub struct VisionSignatureObj {
    base: ObjBase,
    signature: VisionSignature,
}

#[class_methods]
impl VisionSignatureObj {
    pub fn new(signature: VisionSignature) -> Self {
        Self {
            signature,
            base: Self::OBJ_TYPE.into(),
        }
    }

    pub fn signature(&self) -> VisionSignature {
        self.signature
    }

    /// Creates a `VisionSignature`.
    ///
    /// `u_min`, `u_max`, and `u_mean` are the minimum, maximum, and mean values on the U axis. `v_min`,
    /// `v_max`, and `v_mean` are the corresponding values on the V axis. `range` is the detection range
    /// scale factor. All seven arguments are positional-only; `flags` is set to 0.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// my_signature = VisionSignature(10049, 11513, 10781, -425, 1, -212, 4.1)
    /// ```
    ///
    /// # Raises
    ///
    /// - `TypeError`: If a threshold is not an integer, `range` is not numeric, a keyword argument is
    ///   supplied, or the argument count is not exactly seven.
    #[make_new]
    #[stub(
        sig = "(self, u_min: int, u_max: int, u_mean: int, v_min: int, v_max: int, v_mean: int, range: float, /) -> None"
    )]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(7, 7).assert_nkw(0, 0);

        let u_min = reader.next_positional()?;
        let u_max = reader.next_positional()?;
        let u_mean = reader.next_positional()?;

        let v_min = reader.next_positional()?;
        let v_max = reader.next_positional()?;
        let v_mean = reader.next_positional()?;

        let range = reader.next_positional()?;

        Ok(Self {
            base: ObjBase::new(ty),
            signature: VisionSignature::new((u_min, u_max, u_mean), (v_min, v_max, v_mean), range),
        })
    }

    #[attr]
    #[stub(attrs = [
        "u_min: int",
        "u_max: int",
        "u_mean: int",
        "v_min: int",
        "v_max: int",
        "v_mean: int",
        "range: float",
        "flags: int",
    ])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "u_min" => self.signature.u_threshold.0.into(),
            "u_max" => self.signature.u_threshold.1.into(),
            "u_mean" => self.signature.u_threshold.2.into(),

            "v_min" => self.signature.v_threshold.0.into(),
            "v_max" => self.signature.v_threshold.1.into(),
            "v_mean" => self.signature.v_threshold.2.into(),

            "range" => self.signature.range.into(),
            "flags" => Obj::from(self.signature.flags as i32),
            _ => return,
        })
    }

    fn eq(lhs: &Self, rhs: &Self) -> bool {
        lhs.signature.u_threshold == rhs.signature.u_threshold
            && lhs.signature.v_threshold == rhs.signature.v_threshold
            && lhs.signature.range == rhs.signature.range
            && lhs.signature.flags == rhs.signature.flags
    }

    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(
                rhs.try_as_obj::<Self>()
                    .is_some_and(|rhs| Self::eq(lhs, rhs)),
            ),
            _ => Obj::NULL,
        }
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        let _ = write!(
            print,
            "VisionSignature(u_min={}, u_max={}, u_mean={}, v_min={}, v_max={}, v_mean={}, range={}, flags=0x{:02x})",
            self.signature.u_threshold.0,
            self.signature.u_threshold.1,
            self.signature.u_threshold.2,
            self.signature.v_threshold.0,
            self.signature.v_threshold.1,
            self.signature.v_threshold.2,
            self.signature.range,
            self.signature.flags,
        );
    }
}
