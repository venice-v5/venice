use std::sync::atomic::{AtomicU64, Ordering};

/// Tracks the currently active acquisition of a reusable registry slot.
///
/// Zero is reserved for "inactive". A deferred operation may act only while the generation it
/// captured is still active, so freeing and reacquiring a slot invalidates every older operation.
pub struct GenerationTracker {
    active: AtomicU64,
    next: AtomicU64,
}

impl GenerationTracker {
    pub const fn new() -> Self {
        Self {
            active: AtomicU64::new(0),
            next: AtomicU64::new(0),
        }
    }

    pub fn activate(&self) -> u64 {
        let previous = self
            .next
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |generation| {
                generation.checked_add(1)
            })
            .expect("registry generation space exhausted");
        let generation = previous + 1;
        self.active.store(generation, Ordering::Release);
        generation
    }

    pub fn deactivate(&self, generation: u64) {
        let _ = self
            .active
            .compare_exchange(generation, 0, Ordering::AcqRel, Ordering::Acquire);
    }

    pub fn is_active(&self, generation: u64) -> bool {
        generation != 0 && self.active.load(Ordering::Acquire) == generation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdiPortIdentity {
    pub number: u8,
    pub expander_number: Option<u8>,
}

impl AdiPortIdentity {
    pub const fn new(number: u8, expander_number: Option<u8>) -> Self {
        Self {
            number,
            expander_number,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdiPortPlanError {
    Unavailable(usize),
    Duplicate,
}

/// Validates an ADI reservation plan without mutating any port state.
///
/// Callers may consume the ports only after this succeeds and after every other constructor
/// argument and relationship has already been validated.
pub fn validate_adi_port_plan<const N: usize>(
    ports: [(AdiPortIdentity, bool); N],
) -> Result<(), AdiPortPlanError> {
    for (index, (_, available)) in ports.iter().enumerate() {
        if !available {
            return Err(AdiPortPlanError::Unavailable(index));
        }
    }

    for index in 0..N {
        if ports[index + 1..]
            .iter()
            .any(|(identity, _)| *identity == ports[index].0)
        {
            return Err(AdiPortPlanError::Duplicate);
        }
    }

    Ok(())
}
