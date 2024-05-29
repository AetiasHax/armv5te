#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused)]
// Generated by armv5te-generator. Do not edit!
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Argument {
    #[default]
    None,
    /// General-purpose register
    Reg(Reg),
    /// List of general-purpose registers
    RegList(RegList),
    /// Coprocessor register
    CoReg(CoReg),
    /// Status register
    StatusReg(StatusReg),
    /// Status register mask
    StatusMask(StatusMask),
    /// Shift operation
    Shift(Shift),
    /// Immediate shift offset
    ShiftImm(ShiftImm),
    /// Register shift offset
    ShiftReg(ShiftReg),
    /// Unsigned immediate
    UImm(u32),
    /// Signed immediate
    SImm(i32),
    /// Signed immediate offset
    OffsetImm(OffsetImm),
    /// Register offset
    OffsetReg(OffsetReg),
    /// Branch destination offset
    BranchDest(i32),
    /// Additional inStruction options for coprocessor
    CoOption(u32),
    /// Coprocessor operation to perform (user-defined)
    CoOpcode(u32),
    /// Coprocessor number
    CoprocNum(u32),
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Register {
    Illegal = u8::MAX,
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    /// Frame Pointer
    Fp = 11,
    /// Intra Procedure call scratch register
    Ip = 12,
    /// Stack Pointer
    Sp = 13,
    /// Link Register
    Lr = 14,
    /// Program Counter
    Pc = 15,
}
impl Register {
    pub fn parse(value: u32) -> Self {
        if value <= 15 {
            unsafe { std::mem::transmute::<u8, Self>(value as u8) }
        } else {
            Self::Illegal
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum StatusReg {
    Illegal = u8::MAX,
    Cpsr = 0,
    Spsr = 1,
}
impl StatusReg {
    pub fn parse(value: u32) -> Self {
        if value <= 1 {
            unsafe { std::mem::transmute::<u8, Self>(value as u8) }
        } else {
            Self::Illegal
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Shift {
    Illegal = u8::MAX,
    /// Logical shift left
    Lsl = 0,
    /// Logical shift right
    Lsr = 1,
    /// Arithmetic shift right
    Asr = 2,
    /// Rotate right
    Ror = 3,
    /// Rotate right and extend
    Rrx = 4,
}
impl Shift {
    pub fn parse(value: u32) -> Self {
        if value <= 4 {
            unsafe { std::mem::transmute::<u8, Self>(value as u8) }
        } else {
            Self::Illegal
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Reg {
    /// Use as base register
    pub deref: bool,
    /// Register
    pub reg: Register,
    /// When used as a base register, update this register's value
    pub writeback: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RegList {
    /// Bitfield of registers
    pub regs: u32,
    /// Access user-mode registers from elevated mode
    pub user_mode: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum CoReg {
    Illegal = u8::MAX,
    C0 = 0,
    C1 = 1,
    C2 = 2,
    C3 = 3,
    C4 = 4,
    C5 = 5,
    C6 = 6,
    C7 = 7,
    C8 = 8,
    C9 = 9,
    C10 = 10,
    C11 = 11,
    C12 = 12,
    C13 = 13,
    C14 = 14,
    C15 = 15,
}
impl CoReg {
    pub fn parse(value: u32) -> Self {
        if value <= 15 {
            unsafe { std::mem::transmute::<u8, Self>(value as u8) }
        } else {
            Self::Illegal
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StatusMask {
    /// Control field mask (c)
    pub control: bool,
    /// Extension field mask (x)
    pub extension: bool,
    /// Flags field mask (f)
    pub flags: bool,
    /// Status register
    pub reg: StatusReg,
    /// Status field mask (s)
    pub status: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ShiftImm {
    /// Immediate shift offset
    pub imm: u32,
    /// Shift operation
    pub op: Shift,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ShiftReg {
    /// Shift operation
    pub op: Shift,
    /// Register shift offset
    pub reg: Register,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct OffsetImm {
    /// If true, add the offset to the base register and write-back AFTER derefencing the base register
    pub post_indexed: bool,
    /// Offset value
    pub value: i32,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct OffsetReg {
    /// If true, add the offset to the base register, otherwise subtract
    pub add: bool,
    /// If true, add the offset to the base register and write-back AFTER derefencing the base register
    pub post_indexed: bool,
    /// Offset value
    pub reg: Register,
}
