#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct MeaninglessBytes<const NUM_BYTES: usize> {
    bytes: [u8; NUM_BYTES],
}

// This sure seems sketchy - the derive macro refuses to do this, but I can't think what's bad about it.
unsafe impl<const NUM_BYTES: usize> bytemuck::NoUninit for MeaninglessBytes<NUM_BYTES> { }

impl<const NUM_BYTES: usize> Default for MeaninglessBytes<NUM_BYTES> {
    fn default() -> Self {
        Self { bytes: [0; NUM_BYTES] }
    }
}