use crate::{
    interpreter_types::{InterpreterTypes as ITy, RuntimeFlag, StackTr},
    Host, InstructionExecResult as Result,
};
use primitives::hardfork::SpecId::*;

use crate::InstructionContext as Ictx;

/// EIP-1344: ChainID opcode
pub fn chainid<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    check!(context.interpreter, ISTANBUL);
    push!(context.interpreter, context.host.chain_id());
    Ok(())
}

/// Implements the COINBASE instruction.
///
/// Pushes the current block's beneficiary address onto the stack.
pub fn coinbase<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    push!(
        context.interpreter,
        context.host.beneficiary().into_word().into()
    );
    Ok(())
}

/// Implements the TIMESTAMP instruction.
///
/// Pushes the current block's timestamp onto the stack.
pub fn timestamp<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    push!(context.interpreter, context.host.timestamp());
    Ok(())
}

/// Implements the NUMBER instruction.
///
/// Pushes the current block number onto the stack.
pub fn block_number<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    push!(context.interpreter, context.host.block_number());
    Ok(())
}

/// Implements the DIFFICULTY/PREVRANDAO instruction.
///
/// Pushes the block difficulty (pre-merge) or prevrandao (post-merge) onto the stack.
pub fn difficulty<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    if context
        .interpreter
        .runtime_flag
        .spec_id()
        .is_enabled_in(MERGE)
    {
        // Unwrap is safe as this fields is checked in validation handler.
        push!(context.interpreter, context.host.prevrandao().unwrap());
    } else {
        push!(context.interpreter, context.host.difficulty());
    }
    Ok(())
}

/// Implements the GASLIMIT instruction.
///
/// Pushes the current block's gas limit onto the stack.
pub fn gaslimit<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    push!(context.interpreter, context.host.gas_limit());
    Ok(())
}

/// EIP-3198: BASEFEE opcode
pub fn basefee<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    check!(context.interpreter, LONDON);
    push!(context.interpreter, context.host.basefee());
    Ok(())
}

/// EIP-7516: BLOBBASEFEE opcode
pub fn blob_basefee<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    check!(context.interpreter, CANCUN);
    push!(context.interpreter, context.host.blob_gasprice());
    Ok(())
}

/// EIP-7843: SLOTNUM opcode
pub fn slot_num<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    check_amsterdam_opcodes!(context);
    push!(context.interpreter, context.host.slot_num());
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        host::DummyHost,
        instructions::{gas_table, instruction_table},
        interpreter::{EthInterpreter, ExtBytecode, InputsImpl, SharedMemory},
        InstructionResult, Interpreter, InterpreterAction,
    };
    use bytecode::opcode::SLOTNUM;
    use bytecode::Bytecode;
    use primitives::{hardfork::SpecId, Bytes, U256};

    fn run_slot_num(spec: SpecId, enable_amsterdam_opcodes: bool) -> Interpreter {
        let bytecode = Bytecode::new_raw(Bytes::copy_from_slice(&[SLOTNUM]));
        let mut interpreter = Interpreter::<EthInterpreter>::new(
            SharedMemory::new(),
            ExtBytecode::new(bytecode),
            InputsImpl::default(),
            false,
            spec,
            u64::MAX,
        );
        let table = instruction_table::<EthInterpreter, DummyHost>();
        let gas = gas_table();
        let mut host = DummyHost::new(spec).with_amsterdam_opcodes(enable_amsterdam_opcodes);
        interpreter.run_plain(&table, &gas, &mut host);
        interpreter
    }

    fn run_slot_num_action(spec: SpecId, enable_amsterdam_opcodes: bool) -> InterpreterAction {
        let bytecode = Bytecode::new_raw(Bytes::copy_from_slice(&[SLOTNUM]));
        let mut interpreter = Interpreter::<EthInterpreter>::new(
            SharedMemory::new(),
            ExtBytecode::new(bytecode),
            InputsImpl::default(),
            false,
            spec,
            u64::MAX,
        );
        let table = instruction_table::<EthInterpreter, DummyHost>();
        let gas = gas_table();
        let mut host = DummyHost::new(spec).with_amsterdam_opcodes(enable_amsterdam_opcodes);
        interpreter.run_plain(&table, &gas, &mut host)
    }

    #[test]
    fn test_slot_num_not_activated_on_osaka_when_switch_off() {
        let action = run_slot_num_action(SpecId::OSAKA, false);
        assert_eq!(
            action.instruction_result(),
            Some(InstructionResult::NotActivated)
        );
    }

    #[test]
    fn test_slot_num_on_osaka_with_switch_matches_amsterdam() {
        let with_switch = run_slot_num(SpecId::OSAKA, true);
        let on_amsterdam = run_slot_num(SpecId::AMSTERDAM, false);
        assert_eq!(with_switch.stack.data(), on_amsterdam.stack.data());
        assert_eq!(with_switch.stack.data()[0], U256::ZERO);
    }
}
