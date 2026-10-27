//! The size of a frame result and of the values it is built from.

use interpreter::{CallOutcome, CreateOutcome, Gas, InterpreterResult};
use revm_handler::FrameResult;

/// A frame result is 208 bytes on a 64-bit target, and the values it is built from are what that
/// takes. The handler moves frame results through its wrappers several times per frame, and past
/// about 256 bytes each move is a library copy rather than an inline one, so a field added to any
/// of them is measured before this changes.
#[cfg(target_pointer_width = "64")]
#[test]
fn test_a_frame_result_is_208_bytes() {
    use core::mem::size_of;
    assert_eq!(size_of::<Gas>(), 104);
    assert_eq!(size_of::<InterpreterResult>(), 144);
    assert_eq!(size_of::<CallOutcome>(), 208);
    assert_eq!(size_of::<CreateOutcome>(), 192);
    assert_eq!(size_of::<FrameResult>(), 208);
}
