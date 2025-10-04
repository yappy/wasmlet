use anyhow::{Context, Ok};
use bytes::Buf;

// https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-6.html

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Op {
    /// Do nothing.
    Nop,
    /// Push null.
    /// ... -> ..., null
    AconstNull,
    IconstM1,
    Iconst0,
    Iconst1,
    Iconst2,
    Iconst3,
    Iconst4,
    Iconst5,
    Lconst0,
    Lconst1,
    Fconst0,
    Fconst1,
    Fconst2,
    Dconst0,
    Dconst1,
    Bipush {
        byte: u8,
    },
    Sipush {
        bytes: u16,
    },
    Ldc {
        index: u8,
    },
    LdcW {
        index: u16,
    },
    Ldc2W {
        index: u16,
    },
    Iload {
        index: u8,
    },
    Lload {
        index: u8,
    },
    Fload {
        index: u8,
    },
    Dload {
        index: u8,
    },
    Aload {
        index: u8,
    },
    Iload0,
    Iload1,
    Iload2,
    Iload3,
    Lload0,
    Lload1,
    Lload2,
    Lload3,
    Fload0,
    Fload1,
    Fload2,
    Fload3,
    Dload0,
    Dload1,
    Dload2,
    Dload3,
    Aload0,
    Aload1,
    Aload2,
    Aload3,
    Iaload,
    Laload,
    Faload,
    Daload,
    Aaload,
    Baload,
    Caload,
    Saload,
    Istore {
        index: u8,
    },
    Lstore {
        index: u8,
    },
    Fstore {
        index: u8,
    },
    Dstore {
        index: u8,
    },
    Astore {
        index: u8,
    },
    Istore0,
    Istore1,
    Istore2,
    Istore3,
    Lstore0,
    Lstore1,
    Lstore2,
    Lstore3,
    Fstore0,
    Fstore1,
    Fstore2,
    Fstore3,
    Dstore0,
    Dstore1,
    Dstore2,
    Dstore3,
    Astore0,
    Astore1,
    Astore2,
    Astore3,
    Iastore,
    Lastore,
    Fastore,
    Dastore,
    Aastore,
    Bastore,
    Castore,
    Sastore,
    Pop,
    Pop2,
    Dup,
    DupX1,
    DupX2,
    Dup2,
    Dup2X1,
    Dup2X2,
    Swap,
    Iadd,
    Ladd,
    Fadd,
    Dadd,
    Isub,
    Lsub,
    Fsub,
    Dsub,
    Imul,
    Lmul,
    Fmul,
    Dmul,
    Idiv,
    Ldiv,
    Fdiv,
    Ddiv,
    Irem,
    Lrem,
    Frem,
    Drem,
    Ineg,
    Lneg,
    Fneg,
    Dneg,
    Ishl,
    Lshl,
    Ishr,
    Lshr,
    Iushr,
    Lushr,
    Iand,
    Land,
    Ior,
    Lor,
    Ixor,
    Lxor,
    Iinc {
        index: u8,
        constant: i8,
    },
    I2L,
    I2F,
    I2D,
    L2I,
    L2F,
    L2D,
    F2I,
    F2L,
    F2D,
    D2I,
    D2L,
    D2F,
    I2B,
    I2C,
    I2S,
    Lcmp,
    Fcmpl,
    Fcmpg,
    Dcmpl,
    Dcmpg,
    Ifeq {
        branch: i16,
    },
    Ifne {
        branch: i16,
    },
    Iflt {
        branch: i16,
    },
    Ifge {
        branch: i16,
    },
    Ifgt {
        branch: i16,
    },
    Ifle {
        branch: i16,
    },
    IfIcmpeq {
        branch: i16,
    },
    IfIcmpne {
        branch: i16,
    },
    IfIcmplt {
        branch: i16,
    },
    IfIcmpge {
        branch: i16,
    },
    IfIcmpgt {
        branch: i16,
    },
    IfIcmple {
        branch: i16,
    },
    IfAcmpeq {
        branch: i16,
    },
    IfAcmpne {
        branch: i16,
    },
    Goto {
        branch: i16,
    },
    Jsr {
        branch: i16,
    },
    Ret {
        index: u8,
    },
    Tableswitch {
        default: i32,
        low: i32,
        high: i32,
        jump_offsets: Vec<i32>,
    },
    Lookupswitch {
        default: i32,
        npairs: i32,
        match_offsets: Vec<(i32, i32)>,
    },
    Ireturn,
    Lreturn,
    Freturn,
    Dreturn,
    Areturn,
    Return,
    Getstatic {
        index: u16,
    },
    Putstatic {
        index: u16,
    },
    Getfield {
        index: u16,
    },
    Putfield {
        index: u16,
    },
    Invokevirtual {
        index: u16,
    },
    /// Invoke instance method; special handling for superclass, private,
    /// and instance initialization method invocations.
    /// objectref, [arg1, [arg2 ...]] -> ...
    Invokespecial {
        /// cp[index] must be a method reference
        index: u16,
    },
    Invokestatic {
        index: u16,
    },
    Invokeinterface {
        index: u16,
        count: u8,
    },
    Invokedynamic {
        index: u16,
    },
    New {
        index: u16,
    },
    Newarray {
        atype: u8,
    },
    Anewarray {
        index: u16,
    },
    Arraylength,
    Athrow,
    Checkcast {
        index: u16,
    },
    Instanceof {
        index: u16,
    },
    Monitorenter,
    Monitorexit,
    Wide {
        modified_opcode: Box<Op>,
    },
    Multianewarray {
        index: u16,
        dimensions: u8,
    },
    Ifnull {
        branch: i16,
    },
    Ifnonnull {
        branch: i16,
    },
    GotoW {
        branch: i32,
    },
    JsrW {
        branch: i32,
    },
}

pub fn next_op(mut bcode: &[u8]) -> anyhow::Result<(Op, &[u8])> {
    let opcode = bcode.try_get_u8().context("invalid pc")?;

    let op = match opcode {
        0x00 => Op::Nop,
        0x01 => Op::AconstNull,
        0x10 => Op::Bipush {
            byte: bcode.try_get_u8().context("invalid op")?,
        },
        0x2a => Op::Aload0,
        0xb1 => Op::Return,
        0xb5 => {
            let index = bcode.try_get_u16().context("invalid op")?;
            Op::Getstatic { index }
        }
        0xb7 => {
            let index = bcode.try_get_u16().context("invalid op")?;
            Op::Invokespecial { index }
        }
        _ => anyhow::bail!("unsupported opcode: 0x{opcode:02x}"),
    };

    Ok((op, bcode))
}
