// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module lays out the basic abstract costing schedule for bytecode instructions and for the
//! native functions.

use alloc::vec;
use lazy_static::lazy_static;
use move_binary_format::{
    file_format::{
        Bytecode::*, ConstantPoolIndex, FieldHandleIndex, FieldInstantiationIndex,
        FunctionHandleIndex, FunctionInstantiationIndex, SignatureIndex,
        StructDefInstantiationIndex, StructDefinitionIndex,
    },
    file_format_common::instruction_key,
};
use move_core_types::u256;
use move_stdlib::natives::GasParameters;
use move_vm_test_utils::gas_schedule::{new_from_instructions, CostTable, GasCost};

// TODO(rqnsom): tweak the cost
/// A predefined gas cost to published byte ratio.
pub const GAS_COST_PER_PUBLISHED_BYTE: u64 = 100;

lazy_static! {
    // TODO(rqnsom): tweak the cost for intructions
    /// A predefined gas strategy for instruction table cost.
    pub static ref INSTRUCTION_COST_TABLE: CostTable = {
        let mut instrs = vec![
        (MoveTo(StructDefinitionIndex::new(0)), GasCost::new(13, 1)),
        (
            MoveToGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(27, 1),
        ),
        (
            MoveFrom(StructDefinitionIndex::new(0)),
            GasCost::new(459, 1),
        ),
        (
            MoveFromGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(13, 1),
        ),
        (BrTrue(0), GasCost::new(1, 1)),
        (WriteRef, GasCost::new(1, 1)),
        (Mul, GasCost::new(1, 1)),
        (MoveLoc(0), GasCost::new(1, 1)),
        (And, GasCost::new(1, 1)),
        (Pop, GasCost::new(1, 1)),
        (BitAnd, GasCost::new(2, 1)),
        (ReadRef, GasCost::new(1, 1)),
        (Sub, GasCost::new(1, 1)),
        (MutBorrowField(FieldHandleIndex::new(0)), GasCost::new(1, 1)),
        (
            MutBorrowFieldGeneric(FieldInstantiationIndex::new(0)),
            GasCost::new(1, 1),
        ),
        (ImmBorrowField(FieldHandleIndex::new(0)), GasCost::new(1, 1)),
        (
            ImmBorrowFieldGeneric(FieldInstantiationIndex::new(0)),
            GasCost::new(1, 1),
        ),
        (Add, GasCost::new(1, 1)),
        (CopyLoc(0), GasCost::new(1, 1)),
        (StLoc(0), GasCost::new(1, 1)),
        (Ret, GasCost::new(638, 1)),
        (Lt, GasCost::new(1, 1)),
        (LdU8(0), GasCost::new(1, 1)),
        (LdU64(0), GasCost::new(1, 1)),
        (LdU128(0), GasCost::new(1, 1)),
        (CastU8, GasCost::new(2, 1)),
        (CastU64, GasCost::new(1, 1)),
        (CastU128, GasCost::new(1, 1)),
        (Abort, GasCost::new(1, 1)),
        (MutBorrowLoc(0), GasCost::new(2, 1)),
        (ImmBorrowLoc(0), GasCost::new(1, 1)),
        (LdConst(ConstantPoolIndex::new(0)), GasCost::new(1, 1)),
        (Ge, GasCost::new(1, 1)),
        (Xor, GasCost::new(1, 1)),
        (Shl, GasCost::new(2, 1)),
        (Shr, GasCost::new(1, 1)),
        (Neq, GasCost::new(1, 1)),
        (Not, GasCost::new(1, 1)),
        (Call(FunctionHandleIndex::new(0)), GasCost::new(1132, 1)),
        (
            CallGeneric(FunctionInstantiationIndex::new(0)),
            GasCost::new(582, 1),
        ),
        (Le, GasCost::new(2, 1)),
        (Branch(0), GasCost::new(1, 1)),
        (Unpack(StructDefinitionIndex::new(0)), GasCost::new(2, 1)),
        (
            UnpackGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(2, 1),
        ),
        (Or, GasCost::new(2, 1)),
        (LdFalse, GasCost::new(1, 1)),
        (LdTrue, GasCost::new(1, 1)),
        (Mod, GasCost::new(1, 1)),
        (BrFalse(0), GasCost::new(1, 1)),
        (Exists(StructDefinitionIndex::new(0)), GasCost::new(41, 1)),
        (
            ExistsGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(34, 1),
        ),
        (BitOr, GasCost::new(2, 1)),
        (FreezeRef, GasCost::new(1, 1)),
        (
            MutBorrowGlobal(StructDefinitionIndex::new(0)),
            GasCost::new(21, 1),
        ),
        (
            MutBorrowGlobalGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(15, 1),
        ),
        (
            ImmBorrowGlobal(StructDefinitionIndex::new(0)),
            GasCost::new(23, 1),
        ),
        (
            ImmBorrowGlobalGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(14, 1),
        ),
        (Div, GasCost::new(3, 1)),
        (Eq, GasCost::new(1, 1)),
        (Gt, GasCost::new(1, 1)),
        (Pack(StructDefinitionIndex::new(0)), GasCost::new(2, 1)),
        (
            PackGeneric(StructDefInstantiationIndex::new(0)),
            GasCost::new(2, 1),
        ),
        (Nop, GasCost::new(1, 1)),
        (VecPack(SignatureIndex::new(0), 0), GasCost::new(84, 1)),
        (VecLen(SignatureIndex::new(0)), GasCost::new(98, 1)),
        (VecImmBorrow(SignatureIndex::new(0)), GasCost::new(1334, 1)),
        (VecMutBorrow(SignatureIndex::new(0)), GasCost::new(1902, 1)),
        (VecPushBack(SignatureIndex::new(0)), GasCost::new(53, 1)),
        (VecPopBack(SignatureIndex::new(0)), GasCost::new(227, 1)),
        (VecUnpack(SignatureIndex::new(0), 0), GasCost::new(572, 1)),
        (VecSwap(SignatureIndex::new(0)), GasCost::new(1436, 1)),
        (LdU16(0), GasCost::new(1, 1)),
        (LdU32(0), GasCost::new(1, 1)),
        (LdU256(u256::U256::zero()), GasCost::new(1, 1)),
        (CastU16, GasCost::new(2, 1)),
        (CastU32, GasCost::new(2, 1)),
        (CastU256, GasCost::new(2, 1)),
    ];

        // Note that the DiemVM is expecting the table sorted by instruction order.
        instrs.sort_by_key(|cost| instruction_key(&cost.0));

        new_from_instructions(instrs)
    };
}

lazy_static! {
    // TODO(rqnsom): tweak the cost for native parameter
    /// A predefined gas strategy for native functions.
    pub static ref NATIVE_COST_PARAMS: GasParameters = {
        GasParameters {
            bcs: move_stdlib::natives::bcs::GasParameters {
                to_bytes: move_stdlib::natives::bcs::ToBytesGasParameters {
                    per_byte_serialized: 1000.into(),
                    legacy_min_output_size: 1000.into(),
                    failure: 1000.into(),
                },
            },

            hash: move_stdlib::natives::hash::GasParameters {
                sha2_256: move_stdlib::natives::hash::Sha2_256GasParameters {
                    base: 1000.into(),
                    per_byte: 1000.into(),
                    legacy_min_input_len: 1000.into(),
                },
                sha3_256: move_stdlib::natives::hash::Sha3_256GasParameters {
                    base: 1000.into(),
                    per_byte: 1000.into(),
                    legacy_min_input_len: 1000.into(),
                },
            },
            type_name: move_stdlib::natives::type_name::GasParameters {
                get: move_stdlib::natives::type_name::GetGasParameters {
                    base: 1000.into(),
                    per_byte: 1000.into(),
                },
            },
            signer: move_stdlib::natives::signer::GasParameters {
                borrow_address: move_stdlib::natives::signer::BorrowAddressGasParameters { base: 1000.into() },
            },
            string: move_stdlib::natives::string::GasParameters {
                check_utf8: move_stdlib::natives::string::CheckUtf8GasParameters {
                    base: 1000.into(),
                    per_byte: 1000.into(),
                },
                is_char_boundary: move_stdlib::natives::string::IsCharBoundaryGasParameters { base: 1000.into() },
                sub_string: move_stdlib::natives::string::SubStringGasParameters {
                    base: 1000.into(),
                    per_byte: 1000.into(),
                },
                index_of: move_stdlib::natives::string::IndexOfGasParameters {
                    base: 1000.into(),
                    per_byte_pattern: 1000.into(),
                    per_byte_searched: 1000.into(),
                },
            },
            vector: move_stdlib::natives::vector::GasParameters {
                empty: move_stdlib::natives::vector::EmptyGasParameters { base: 1000.into() },
                length: move_stdlib::natives::vector::LengthGasParameters { base: 1000.into() },
                push_back: move_stdlib::natives::vector::PushBackGasParameters {
                    base: 1000.into(),
                    legacy_per_abstract_memory_unit: 1000.into(),
                },
                borrow: move_stdlib::natives::vector::BorrowGasParameters { base: 1000.into() },
                pop_back: move_stdlib::natives::vector::PopBackGasParameters { base: 1000.into() },
                destroy_empty: move_stdlib::natives::vector::DestroyEmptyGasParameters { base: 1000.into() },
                swap: move_stdlib::natives::vector::SwapGasParameters { base: 1000.into() },
            },
            #[cfg(feature = "testing")]
            unit_test: move_stdlib::natives::unit_test::GasParameters {
                create_signers_for_testing: move_stdlib::natives::unit_test::CreateSignersForTestingGasParameters {
                    base_cost: 1000.into(),
                    unit_cost: 1000.into(),
                },
            },
        }
    };
}
