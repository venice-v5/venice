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

    #[make_new]
    #[stub(
        sig = "(self, u_min: int, u_max: int, u_mean: int, v_min: int, v_max: int, v_mean: int, range: float) -> None"
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
            BinaryOpCode::Equal => Obj::from_bool(Self::eq(lhs, rhs.as_obj::<Self>())),
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
