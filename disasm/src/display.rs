use std::fmt::{self, Display, Formatter};

use crate::{
    args::{
        Argument, CoReg, CpsrFlags, CpsrMode, Endian, OffsetImm, OffsetReg, Reg, Register, Shift, ShiftImm, ShiftReg,
        StatusMask, StatusReg,
    },
    parse::ParsedIns,
};

impl ParsedIns {
    pub fn display(&self, options: DisplayOptions) -> ParsedInsDisplay<'_> {
        ParsedInsDisplay { ins: self, options }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct DisplayOptions {
    pub reg_names: RegNames,
}

pub struct ParsedInsDisplay<'a> {
    ins: &'a ParsedIns,
    options: DisplayOptions,
}

impl<'a> Display for ParsedInsDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ins.mnemonic)?;
        if self.ins.args[0] != Argument::None {
            write!(f, " ")?;
        }
        let mut comma = false;
        let mut deref = false;
        let mut writeback = false;
        for arg in self.ins.args_iter() {
            if deref {
                match arg {
                    Argument::OffsetImm(OffsetImm {
                        post_indexed: true,
                        value: _,
                    })
                    | Argument::OffsetReg(OffsetReg {
                        add: _,
                        post_indexed: true,
                        reg: _,
                    })
                    | Argument::CoOption(_) => {
                        deref = false;
                        write!(f, "]")?;
                        if writeback {
                            write!(f, "!")?;
                            writeback = false;
                        }
                    }
                    _ => {}
                }
            }
            if comma {
                write!(f, ", ")?;
            }
            if let Argument::Reg(Reg {
                deref: true,
                reg,
                writeback: wb,
            }) = arg
            {
                deref = true;
                writeback = *wb;
                write!(f, "[{}", reg.display(self.options.reg_names))?;
            } else {
                write!(f, "{}", arg.display(self.options))?;
            }
            comma = true;
        }
        if deref {
            write!(f, "]")?;
            if writeback {
                write!(f, "!")?;
            }
        }
        Ok(())
    }
}

pub struct SignedHex(i32);

impl Display for SignedHex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "#")?;
        if self.0.is_negative() {
            write!(f, "-")?;
        }
        write!(f, "0x{:x}", self.0.abs())
    }
}

impl Argument {
    pub fn display(&self, options: DisplayOptions) -> DisplayArgument<'_> {
        DisplayArgument { arg: self, options }
    }
}

pub struct DisplayArgument<'a> {
    arg: &'a Argument,
    options: DisplayOptions,
}

impl<'a> Display for DisplayArgument<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.arg {
            Argument::None => Ok(()),
            Argument::Reg(reg) => {
                write!(f, "{}", reg.reg.display(self.options.reg_names))?;
                if reg.writeback {
                    write!(f, "!")?;
                }
                Ok(())
            }
            Argument::RegList(list) => {
                write!(f, "{{")?;
                let mut first = true;
                for i in 0..16 {
                    if (list.regs & (1 << i)) != 0 {
                        if !first {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", Register::parse(i).display(self.options.reg_names))?;
                        first = false;
                    }
                }
                write!(f, "}}")?;
                if list.user_mode {
                    write!(f, "^")?;
                }
                Ok(())
            }
            Argument::CoReg(x) => write!(f, "{}", x),
            Argument::StatusReg(x) => write!(f, "{}", x),
            Argument::UImm(x) => write!(f, "#0x{:x}", x),
            Argument::SImm(x) => write!(f, "{}", SignedHex(*x)),
            Argument::OffsetImm(x) => write!(f, "{}", SignedHex(x.value)),
            Argument::CoOption(x) => write!(f, "{{0x{:x}}}", x),
            Argument::CoOpcode(x) => write!(f, "#{}", x),
            Argument::CoprocNum(x) => write!(f, "p{}", x),
            Argument::ShiftImm(x) => write!(f, "{}", x),
            Argument::ShiftReg(x) => write!(f, "{}", x.display(self.options.reg_names)),
            Argument::OffsetReg(x) => write!(f, "{}", x.display(self.options.reg_names)),
            Argument::BranchDest(x) => write!(f, "{}", SignedHex(*x)),
            Argument::StatusMask(x) => write!(f, "{}", x),
            Argument::Shift(x) => write!(f, "{}", x),
            Argument::SatImm(x) => write!(f, "#0x{:x}", x),
            Argument::CpsrMode(x) => write!(f, "{}", x),
            Argument::CpsrFlags(x) => write!(f, "{}", x),
            Argument::Endian(x) => write!(f, "{}", x),
        }
    }
}

/// How R9 should be displayed
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum R9Use {
    /// R9 or V6.
    #[default]
    GeneralPurpose,
    /// Position-independent data (PID). If true, R9 will display as SB (static base).
    Pid,
    /// Thread-local storage (TLS). If true, R9 will display as TR (TLS register).
    Tls,
}

/// Customizes the display of register names.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct RegNames {
    /// If true, R0-R3 and R4-R11 will display as A1-A4 and V1-V8.
    pub av_registers: bool,
    /// How R9 should be displayed.
    pub r9_use: R9Use,
    /// If true, R10 will display as SL (stack limit).
    pub explicit_stack_limit: bool,
    /// If true, R11 will display as FP (frame pointer).
    pub frame_pointer: bool,
    /// If true, R12 will display as IP (intra procedure call scratch register). Used for interworking and long branches.
    pub ip: bool,
}

impl Register {
    pub fn display(self, names: RegNames) -> RegDisplay {
        RegDisplay(self, names)
    }
}

pub struct RegDisplay(Register, RegNames);

impl Display for RegDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[rustfmt::skip]
        let s = match self.0 {
            Register::Illegal => todo!(),
            Register::R0 => if self.1.av_registers { "a1" } else { "r0" },
            Register::R1 => if self.1.av_registers { "a2" } else { "r1" },
            Register::R2 => if self.1.av_registers { "a3" } else { "r2" },
            Register::R3 => if self.1.av_registers { "a4" } else { "r3" },
            Register::R4 => if self.1.av_registers { "v1" } else { "r4" },
            Register::R5 => if self.1.av_registers { "v2" } else { "r5" },
            Register::R6 => if self.1.av_registers { "v3" } else { "r6" },
            Register::R7 => if self.1.av_registers { "v4" } else { "r7" },
            Register::R8 => if self.1.av_registers { "v5" } else { "r8" },
            Register::R9 => match self.1.r9_use {
                R9Use::GeneralPurpose => if self.1.av_registers { "v6" } else { "r9" },
                R9Use::Pid => "sb",
                R9Use::Tls => "tr",
            },
            Register::R10 => if self.1.explicit_stack_limit { "sl" } else if self.1.av_registers { "v7" } else { "r10" },
            Register::R11 => if self.1.frame_pointer { "fp" } else if self.1.av_registers { "v8" } else { "r11" },
            Register::R12 => if self.1.ip { "ip" } else { "r12" },
            Register::Sp => "sp",
            Register::Lr => "lr",
            Register::Pc => "pc",
        };
        write!(f, "{}", s)
    }
}

impl Display for CoReg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CoReg::Illegal => write!(f, "<illegal>"),
            CoReg::C0 => write!(f, "c0"),
            CoReg::C1 => write!(f, "c1"),
            CoReg::C2 => write!(f, "c2"),
            CoReg::C3 => write!(f, "c3"),
            CoReg::C4 => write!(f, "c4"),
            CoReg::C5 => write!(f, "c5"),
            CoReg::C6 => write!(f, "c6"),
            CoReg::C7 => write!(f, "c7"),
            CoReg::C8 => write!(f, "c8"),
            CoReg::C9 => write!(f, "c9"),
            CoReg::C10 => write!(f, "c10"),
            CoReg::C11 => write!(f, "c11"),
            CoReg::C12 => write!(f, "c12"),
            CoReg::C13 => write!(f, "c13"),
            CoReg::C14 => write!(f, "c14"),
            CoReg::C15 => write!(f, "c15"),
        }
    }
}

impl Display for StatusReg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StatusReg::Illegal => write!(f, "<illegal>"),
            StatusReg::Cpsr => write!(f, "cpsr"),
            StatusReg::Spsr => write!(f, "spsr"),
        }
    }
}

impl Display for StatusMask {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reg)?;
        if self.flags || self.status || self.extension || self.control {
            write!(f, "_")?;
        }
        if self.flags {
            write!(f, "f")?;
        }
        if self.status {
            write!(f, "s")?;
        }
        if self.extension {
            write!(f, "x")?;
        }
        if self.control {
            write!(f, "c")?;
        }
        Ok(())
    }
}

impl Display for Shift {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Shift::Illegal => write!(f, "<illegal>"),
            Shift::Lsl => write!(f, "lsl"),
            Shift::Lsr => write!(f, "lsr"),
            Shift::Asr => write!(f, "asr"),
            Shift::Ror => write!(f, "ror"),
            Shift::Rrx => write!(f, "rrx"),
        }
    }
}

impl Display for ShiftImm {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} #0x{:x}", self.op, self.imm)
    }
}

impl ShiftReg {
    pub fn display(self, names: RegNames) -> DisplayShiftReg {
        DisplayShiftReg(self, names)
    }
}

pub struct DisplayShiftReg(ShiftReg, RegNames);

impl Display for DisplayShiftReg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.0.op, self.0.reg.display(self.1))
    }
}

impl OffsetReg {
    pub fn display(self, names: RegNames) -> DisplayOffsetReg {
        DisplayOffsetReg(self, names)
    }
}

pub struct DisplayOffsetReg(OffsetReg, RegNames);

impl Display for DisplayOffsetReg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !self.0.add {
            write!(f, "-")?;
        }
        write!(f, "{}", self.0.reg.display(self.1))
    }
}

impl Display for CpsrMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "#0x{:x}", self.mode)?;
        if self.writeback {
            write!(f, "!")?;
        }
        Ok(())
    }
}

impl Display for CpsrFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.a {
            write!(f, "a")?;
        }
        if self.i {
            write!(f, "i")?;
        }
        if self.f {
            write!(f, "f")?;
        }
        if !self.a && !self.i && !self.f {
            write!(f, "none")?;
        }
        Ok(())
    }
}

impl Display for Endian {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Endian::Illegal => write!(f, "<illegal>"),
            Endian::Le => write!(f, "le"),
            Endian::Be => write!(f, "be"),
        }
    }
}
