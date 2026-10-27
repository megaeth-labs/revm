//! The settlement of the transaction's frame into the transaction-level gas, which
//! [`Handler::last_frame_result`](crate::Handler::last_frame_result) makes on mainnet and a chain's
//! own handler makes by calling it.

use crate::{frame::handle_reservoir_remaining_gas, FrameResult};
use context::result::FromStringError;
use context_interface::{context::take_error, ContextTr, Database};
use interpreter::GasTracker;
use std::string::ToString;

/// Settles the transaction's frame, the first frame, into the transaction-level `gas`, every step
/// of [`Handler::last_frame_result`](crate::Handler::last_frame_result) but the last: the copy of
/// `gas` into `frame_result`, which is the caller's.
///
/// The steps, each with the halt rule it takes:
/// - `gas` spends all its regular gas, which was forwarded to the frame, and the frame settles
///   into it as a child settles into its caller ([`handle_reservoir_remaining_gas`]): a failing
///   frame rolls back its state gas and history gas, a halt consumes its regular gas, and what
///   the frame left returns to `gas`, its reservoir added to what `gas` kept, which is nothing,
///   since the frame took it whole;
/// - the EIP-2780 refundable first-frame charge is refunded when no account leaf was created,
///   priced through [`Host::state_gas_charge`](context_interface::Host::state_gas_charge) as it
///   was charged; after a halt the regular gas the refund credited is consumed.
///
/// `gas` holds no reservoir on entry: the first frame took it whole
/// ([`GasTracker::forward_all_to_child`]), and the settlement adds back what the frame left, so
/// gas that still held what the frame was given would count it twice. A debug build asserts it.
///
/// A chain whose handler overrides `last_frame_result` calls this rather than copying the mainnet
/// body, so a step the fork adds here reaches its handler too. It applies its own rules to `gas`
/// after this returns and then copies `gas` into the frame's result, as op-revm's handler does
/// with its pre-Regolith deposit rule.
///
/// Inlined, so `last_frame_result` compiles to the code it had while these steps were its body.
///
/// # Errors
///
/// A failure the context recorded before the EIP-2780 refund is returned first, so a failing
/// refund lookup cannot overwrite it. A failed lookup returns the cause the context recorded, or
/// an error saying the lookup failed.
#[inline(always)]
pub fn settle_first_frame<CTX, ERROR>(
    context: &mut CTX,
    frame_result: &mut FrameResult,
    gas: &mut GasTracker,
) -> Result<(), ERROR>
where
    CTX: ContextTr,
    ERROR: From<<CTX::Db as Database>::Error> + FromStringError,
{
    let instruction_result = frame_result.instruction_result();

    debug_assert!(
        gas.reservoir() == 0,
        "the transaction's gas still holds the reservoir its frame was given; the settlement \
         would count it twice (take it with `GasTracker::forward_all_to_child`)"
    );

    // All regular gas was forwarded to the first frame: consume it on the
    // transaction-level gas; the settle below returns the frame's unused
    // part.
    gas.spend_all();

    // Settle the frame into the transaction-level gas like a parent frame.
    handle_reservoir_remaining_gas(
        instruction_result,
        gas,
        frame_result.gas_mut().tracker_mut(),
    );

    // Refund the EIP-2780 refundable first-frame charge when no account
    // leaf was created, exactly like `EthFrame::return_result` refunds
    // the upfront CALL/CREATE state charges of inner frames.
    if let Some(charge) = frame_result.refundable_state_gas_charge() {
        // A failure the frame already recorded (a failed code deposit lookup, say) is
        // returned first, as `EthFrame::return_result` does for inner frames, so a failing
        // refund lookup cannot overwrite it.
        take_error::<ERROR, _>(context.error())?;
        // Priced through the same hook the charge went through. A failed lookup surfaces as
        // an external error: the hook recorded the cause.
        let Some(charge) = context.state_gas_charge(charge) else {
            take_error::<ERROR, _>(context.error())?;
            return Err(ERROR::from_string(
                "state gas price lookup failed".to_string(),
            ));
        };
        gas.refill_reservoir(charge);
        // Unlike an inner frame's caller, the transaction ends here: an
        // exceptional halt consumes all regular gas, including the
        // spilled portion the refill just credited back to `remaining`.
        if instruction_result.is_halt() {
            gas.spend_all();
        }
    }

    Ok(())
}
