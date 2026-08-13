#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidatedColorCode {
    pub ids: [i16; 7],
    pub len: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCodeValidationError {
    Empty,
    InvalidId { index: usize, id: u8 },
    NonContiguous { index: usize },
}

pub fn narrow_slot_id(id: i32, max: u8) -> Option<u8> {
    if (1..=i32::from(max)).contains(&id) {
        Some(id as u8)
    } else {
        None
    }
}

pub fn validate_color_code(
    values: [Option<u8>; 7],
) -> Result<ValidatedColorCode, ColorCodeValidationError> {
    let mut ids = [0; 7];
    let mut len = 0;
    let mut found_gap = false;

    for (index, value) in values.into_iter().enumerate() {
        match value {
            Some(id) if !(1..=7).contains(&id) => {
                return Err(ColorCodeValidationError::InvalidId { index, id });
            }
            Some(_) if found_gap => {
                return Err(ColorCodeValidationError::NonContiguous { index });
            }
            Some(id) => {
                ids[index] = i16::from(id);
                len += 1;
            }
            None => found_gap = true,
        }
    }

    if len == 0 {
        return Err(ColorCodeValidationError::Empty);
    }

    Ok(ValidatedColorCode { ids, len })
}
