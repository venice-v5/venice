use std::fmt::Write;

use argparse::{Args, ArgsReader, PositionalError};
use micropython_macros::{class, class_methods};
use micropython_rs::{
    obj::{AttrOp, Obj, ObjBase, ObjTrait, ObjType},
    ops::BinaryOpCode,
    print::{Print, PrintKind},
    qstr::Qstr,
};
use vexide_devices::smart::vision::VisionCode;

use crate::modvenice::{Exception, read_only_attr::read_only_attr, vision::SignatureId};

/// A vision detection code.
///
/// This class is root-importable. Codes are a special type of detection signature that group multiple `VisionSignature` objects
/// together. A `VisionCode` can associate 2-5 color signatures together, detecting the resulting
/// object when its color signatures are present close to each other.
///
/// These codes work very similarly to
/// [Pixy2 Color Codes](https://docs.pixycam.com/wiki/doku.php?id=wiki:v2:using_color_codes).
///
/// The read-only `sig1` and `sig2` attributes are required signature IDs from 1 to 7; read-only `sig3`,
/// `sig4`, and `sig5` contain additional IDs or `None`. Codes compare equal when all five slots match
/// and have a readable `VisionCode(...)` representation.
#[class(qstr!(VisionCode))]
#[repr(C)]
pub struct VisionCodeObj {
    base: ObjBase,
    code: VisionCode,
}

#[class_methods]
impl VisionCodeObj {
    pub fn new(code: VisionCode) -> Self {
        Self {
            base: Self::OBJ_TYPE.into(),
            code,
        }
    }

    pub fn code(&self) -> VisionCode {
        self.code
    }

    /// Creates a new vision code.
    ///
    /// Two signatures, `sig1` and `sig2`, are required to create a vision code, with an additional three
    /// optional signatures, `sig3`, `sig4`, and `sig5`. Each supplied ID must be an integer from 1 to 7.
    /// Although the generated annotation includes `None` for the trailing slots, the runtime accepts
    /// omission rather than an explicitly passed `None`. All arguments are positional-only.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// # Create a vision code associated with signatures 1, 2, and 3.
    /// code = VisionCode(1, 2, 3)
    /// ```
    ///
    /// # Raises
    ///
    /// - `TypeError`: If a supplied ID is not an integer, `None` is passed explicitly, a keyword argument
    ///   is supplied, or the argument count is outside two to five.
    /// - `ValueError`: If a supplied signature ID is outside the inclusive range 1 to 7.
    #[make_new]
    #[stub(
        sig = "(self, sig1: int, sig2: int, sig3: int | None = None, sig4: int | None = None, sig5: int | None = None) -> None"
    )]
    fn make_new(
        ty: &'static ObjType,
        n_pos: usize,
        n_kw: usize,
        args: &[Obj],
    ) -> Result<Self, Exception> {
        let mut reader = Args::new(n_pos, n_kw, args).reader();
        reader.assert_npos(2, 5).assert_nkw(0, 0);

        let s1 = reader.next_positional::<SignatureId>()?.id();
        let s2 = reader.next_positional::<SignatureId>()?.id();

        fn read_optional_sig(reader: &mut ArgsReader) -> Result<Option<u8>, Exception> {
            let result = reader.next_positional::<SignatureId>();
            match result {
                Ok(id) => Ok(Some(id.id())),
                Err(PositionalError::ArgumentsExhausted) => Ok(None),
                _ => Err(Exception::from(result.unwrap_err())),
            }
        }

        let s3 = read_optional_sig(&mut reader)?;
        let s4 = read_optional_sig(&mut reader)?;
        let s5 = read_optional_sig(&mut reader)?;

        Ok(Self {
            base: ObjBase::new(ty),
            code: VisionCode::new(s1, s2, s3, s4, s5),
        })
    }

    /// Creates a `VisionCode` from a bit representation of its signature IDs.
    ///
    /// The low 15 bits of `id` are interpreted as five three-bit signature slots; zero means an absent
    /// optional slot. This method is intended for packed IDs reported by the sensor and does not validate
    /// that the two required decoded slots are nonzero.
    ///
    /// # Examples
    ///
    /// ```python
    /// from venice import *
    ///
    /// sig_1_id = 1
    /// sig_2_id = 2
    ///
    /// # Store IDs 1 and 2 in the first two of the five three-bit slots.
    /// code_id = (sig_1_id << 12) | (sig_2_id << 9)
    ///
    /// # Create a VisionCode from signatures 1 and 2.
    /// code = VisionCode.from_id(code_id)
    /// ```
    ///
    /// # Raises
    ///
    /// - `TypeError`: If `id` is not an integer.
    #[method(binding = "static")]
    fn from_id(id: i32) -> Self {
        Self::new(VisionCode::from_id(id as u16))
    }

    #[attr]
    #[stub(attrs = [
        "sig1: int",
        "sig2: int",
        "sig3: int | None",
        "sig4: int | None",
        "sig5: int | None",
    ])]
    fn attr(&self, attr: Qstr, op: AttrOp) {
        let AttrOp::Load { result } = op else {
            read_only_attr::<Self>()
        };
        result.return_value(match attr.as_str() {
            "sig1" => Obj::from(self.code.0 as i32),
            "sig2" => (self.code.1 as i32).into(),
            "sig3" => self.code.2.map(i32::from).into(),
            "sig4" => self.code.3.map(i32::from).into(),
            "sig5" => self.code.4.map(i32::from).into(),
            _ => return,
        })
    }

    #[binary_op]
    fn binary_op(op: BinaryOpCode, lhs: &Self, rhs: Obj) -> Obj {
        match op {
            BinaryOpCode::Equal => Obj::from_bool(lhs.code == rhs.as_obj::<Self>().code),
            _ => Obj::NULL,
        }
    }

    #[printer]
    fn printer(&self, print: &mut Print, _kind: PrintKind) {
        let _ = write!(
            print,
            "VisionCode(sig1={}, sig2={}",
            self.code.0, self.code.1
        );
        if let Some(s) = self.code.2 {
            let _ = write!(print, ", sig3={s}");
        }
        if let Some(s) = self.code.3 {
            let _ = write!(print, ", sig4={s}");
        }
        if let Some(s) = self.code.4 {
            let _ = write!(print, ", sig5={s}");
        }
        print.print(")");
    }
}
