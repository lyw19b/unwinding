use core::fmt;
use core::ops;
use gimli::{Register, LoongArch};

use super::maybe_cfi;

// Match DWARF_FRAME_REGISTERS in libgcc
/* Number of hardware registers.  We have:

   - 32 integer registers
   - 32 floating point registers
   - 8 condition code registers
   - 2 fake registers:
        - ARG_POINTER_REGNUM
        - FRAME_POINTER_REGNUM
*/
pub const MAX_REG_RULES: usize = 74;

#[cfg(all(target_feature = "f", not(target_feature = "d")))]
compile_error!("LoongArch with only F extension(single) is not supported");

#[repr(C)]
#[derive(Clone, Default)]
pub struct Context {
    pub gp: [usize; 32],
    pub pc: usize,
    #[cfg(target_feature = "d")]
    pub fp: [usize; 32],
}

impl fmt::Debug for Context {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt = fmt.debug_struct("Context");
        for i in 0..=31 {
            fmt.field(LoongArch::register_name(Register(i as _)).unwrap(), &self.gp[i]);
        }
        #[cfg(target_feature = "d")]
        for i in 0..=31 {
            fmt.field(
                LoongArch::register_name(Register((i + 32) as _)).unwrap(),
                &self.fp[i],
            );
        }
        fmt.finish()
    }
}

impl ops::Index<Register> for Context {
    type Output = usize;

    fn index(&self, reg: Register) -> &usize {
        match reg {
            Register(0..=31) => &self.gp[reg.0 as usize],
            #[cfg(target_feature = "d")]
            Register(32..=63) => &self.fp[(reg.0 - 32) as usize],
            _ => unimplemented!(),
        }
    }
}

impl ops::IndexMut<gimli::Register> for Context {
    fn index_mut(&mut self, reg: Register) -> &mut usize {
        match reg {
            Register(0..=31) => &mut self.gp[reg.0 as usize],
            #[cfg(target_feature = "d")]
            Register(32..=63) => &mut self.fp[(reg.0 - 32) as usize],
            _ => unimplemented!(),
        }
    }
}

macro_rules! code {
    (save_gp) => {
        "
        st.d $r1,  $sp, (8*1 )
        st.d $r2,  $sp, (8*2 )
        st.d $r12, $sp, (8*3 ) # t0 save orign sp
        st.d $r4,  $sp, (8*4 )
        st.d $r5,  $sp, (8*5 )
        st.d $r6,  $sp, (8*6 )
        st.d $r7,  $sp, (8*7 )
        st.d $r8,  $sp, (8*8 )
        st.d $r9,  $sp, (8*9 )
        st.d $r10, $sp, (8*10)
        st.d $r11, $sp, (8*11)
        st.d $r12, $sp, (8*12)
        st.d $r13, $sp, (8*13)
        st.d $r14, $sp, (8*14)
        st.d $r15, $sp, (8*15)
        st.d $r16, $sp, (8*16)
        st.d $r17, $sp, (8*17)
        st.d $r18, $sp, (8*18)
        st.d $r19, $sp, (8*19)
        st.d $r20, $sp, (8*20)
        st.d $r21, $sp, (8*21)
        st.d $r22, $sp, (8*22)
        st.d $r23, $sp, (8*23)
        st.d $r24, $sp, (8*24)
        st.d $r25, $sp, (8*25)
        st.d $r26, $sp, (8*26)
        st.d $r27, $sp, (8*27)
        st.d $r28, $sp, (8*28)
        st.d $r29, $sp, (8*29)
        st.d $r30, $sp, (8*30)
        st.d $r31, $sp, (8*31)
        "
    };
    (save_fp) => {
        // arch option manipulation needed due to LLVM/Rust bug, see rust-lang/rust#80608
        "
        .option push
        .option arch, +d
        fst.d $f0,  $sp, (8 * 33 + 8 * 0)
        fst.d $f1,  $sp, (8 * 33 + 8 * 1)
        fst.d $f2,  $sp, (8 * 33 + 8 * 2)
        fst.d $f3,  $sp, (8 * 33 + 8 * 3)
        fst.d $f4,  $sp, (8 * 33 + 8 * 4)
        fst.d $f5,  $sp, (8 * 33 + 8 * 5)
        fst.d $f6,  $sp, (8 * 33 + 8 * 6)
        fst.d $f7,  $sp, (8 * 33 + 8 * 7)
        fst.d $f8,  $sp, (8 * 33 + 8 * 8)
        fst.d $f9,  $sp, (8 * 33 + 8 * 9)
        fst.d $f10, $sp, (8 * 33 + 8 * 10)
        fst.d $f11, $sp, (8 * 33 + 8 * 11)
        fst.d $f12, $sp, (8 * 33 + 8 * 12)
        fst.d $f13, $sp, (8 * 33 + 8 * 13)
        fst.d $f14, $sp, (8 * 33 + 8 * 14)
        fst.d $f15, $sp, (8 * 33 + 8 * 15)
        fst.d $f16, $sp, (8 * 33 + 8 * 16)
        fst.d $f17, $sp, (8 * 33 + 8 * 17)
        fst.d $f18, $sp, (8 * 33 + 8 * 18)
        fst.d $f19, $sp, (8 * 33 + 8 * 19)
        fst.d $f20, $sp, (8 * 33 + 8 * 20)
        fst.d $f21, $sp, (8 * 33 + 8 * 21)
        fst.d $f22, $sp, (8 * 33 + 8 * 22)
        fst.d $f23, $sp, (8 * 33 + 8 * 23)
        fst.d $f24, $sp, (8 * 33 + 8 * 24)
        fst.d $f25, $sp, (8 * 33 + 8 * 25)
        fst.d $f26, $sp, (8 * 33 + 8 * 26)
        fst.d $f27, $sp, (8 * 33 + 8 * 27)
        fst.d $f28, $sp, (8 * 33 + 8 * 28)
        fst.d $f29, $sp, (8 * 33 + 8 * 29)
        fst.d $f30, $sp, (8 * 33 + 8 * 30)
        fst.d $f31, $sp, (8 * 33 + 8 * 31)
        .option pop
        "
    };
    (restore_gp) => {
        "
        ld.d $r1,  $a0, (8*1 )
        ld.d $r2,  $a0, (8*2 )
        ld.d $r3,  $a0, (8*3 )
        ld.d $r5,  $a0, (8*5 )
        ld.d $r6,  $a0, (8*6 )
        ld.d $r7,  $a0, (8*7 )
        ld.d $r8,  $a0, (8*8 )
        ld.d $r9,  $a0, (8*9 )
        ld.d $r10, $a0, (8*10)
        ld.d $r11, $a0, (8*11)
        ld.d $r12, $a0, (8*12)
        ld.d $r13, $a0, (8*13)
        ld.d $r14, $a0, (8*14)
        ld.d $r15, $a0, (8*15)
        ld.d $r16, $a0, (8*16)
        ld.d $r17, $a0, (8*17)
        ld.d $r18, $a0, (8*18)
        ld.d $r19, $a0, (8*19)
        ld.d $r20, $a0, (8*20)
        ld.d $r21, $a0, (8*21)
        ld.d $r22, $a0, (8*22)
        ld.d $r23, $a0, (8*23)
        ld.d $r24, $a0, (8*24)
        ld.d $r25, $a0, (8*25)
        ld.d $r26, $a0, (8*26)
        ld.d $r27, $a0, (8*27)
        ld.d $r28, $a0, (8*28)
        ld.d $r29, $a0, (8*29)
        ld.d $r30, $a0, (8*30)
        ld.d $r31, $a0, (8*31)
        "
    };
    (restore_fp) => {
        "
        fld.d $f0,  $a0, (8 * 33 + 8 * 0)
        fld.d $f1,  $a0, (8 * 33 + 8 * 1)
        fld.d $f2,  $a0, (8 * 33 + 8 * 2)
        fld.d $f3,  $a0, (8 * 33 + 8 * 3)
        fld.d $f4,  $a0, (8 * 33 + 8 * 4)
        fld.d $f5,  $a0, (8 * 33 + 8 * 5)
        fld.d $f6,  $a0, (8 * 33 + 8 * 6)
        fld.d $f7,  $a0, (8 * 33 + 8 * 7)
        fld.d $f8,  $a0, (8 * 33 + 8 * 8)
        fld.d $f9,  $a0, (8 * 33 + 8 * 9)
        fld.d $f10, $a0, (8 * 33 + 8 * 10)
        fld.d $f11, $a0, (8 * 33 + 8 * 11)
        fld.d $f12, $a0, (8 * 33 + 8 * 12)
        fld.d $f13, $a0, (8 * 33 + 8 * 13)
        fld.d $f14, $a0, (8 * 33 + 8 * 14)
        fld.d $f15, $a0, (8 * 33 + 8 * 15)
        fld.d $f16, $a0, (8 * 33 + 8 * 16)
        fld.d $f17, $a0, (8 * 33 + 8 * 17)
        fld.d $f18, $a0, (8 * 33 + 8 * 18)
        fld.d $f19, $a0, (8 * 33 + 8 * 19)
        fld.d $f20, $a0, (8 * 33 + 8 * 20)
        fld.d $f21, $a0, (8 * 33 + 8 * 21)
        fld.d $f22, $a0, (8 * 33 + 8 * 22)
        fld.d $f23, $a0, (8 * 33 + 8 * 23)
        fld.d $f24, $a0, (8 * 33 + 8 * 24)
        fld.d $f25, $a0, (8 * 33 + 8 * 25)
        fld.d $f26, $a0, (8 * 33 + 8 * 26)
        fld.d $f27, $a0, (8 * 33 + 8 * 27)
        fld.d $f28, $a0, (8 * 33 + 8 * 28)
        fld.d $f29, $a0, (8 * 33 + 8 * 29)
        fld.d $f30, $a0, (8 * 33 + 8 * 30)
        fld.d $f31, $a0, (8 * 33 + 8 * 31)
        "
    };
}

#[naked]
pub extern "C-unwind" fn save_context(f: extern "C" fn(&mut Context, *mut ()), ptr: *mut ()) {
    // No need to save caller-saved registers here.
    #[cfg(target_feature = "d")]
    unsafe {
        core::arch::naked_asm!(
            maybe_cfi!(".cfi_startproc"),
            "
            move   $t0, $sp
            addi.d $sp, $sp, -0x260
            ",
            maybe_cfi!(".cfi_def_cfa_offset 0x260"),
            "st.d  $ra, $sp, 0x250",
            maybe_cfi!(".cfi_offset ra, -16"),
            code!(save_gp),
            code!(save_fp),
            "
            move $t0, $a0
            move $a0, $sp
            jr   $t0
            ld.d   $ra, $sp, 0x250
            addi.d $sp, $sp, 0x260
            ",
            maybe_cfi!(".cfi_def_cfa_offset 0"),
            maybe_cfi!(".cfi_restore ra"),
            "ret",
            maybe_cfi!(".cfi_endproc"),
        );
    }
    #[cfg(not(target_feature = "d"))]
    unsafe {
        core::arch::naked_asm!(
            maybe_cfi!(".cfi_startproc"),
            "
            move   $t0, $sp
            addi.d $sp, $sp, -0x160
            ",
            maybe_cfi!(".cfi_def_cfa_offset 0x160"),
            "sd.d  $ra, $sp, 0x150",
            maybe_cfi!(".cfi_offset ra, -16"),
            code!(save_gp),
            "
            move $t0, $a0
            move $a0, $sp
            jr $t0
            ld.d   $ra, $sp, 0x150
            addi.d $sp, $sp, 0x160
            ",
            maybe_cfi!(".cfi_def_cfa_offset 0"),
            maybe_cfi!(".cfi_restore ra"),
            "ret",
            maybe_cfi!(".cfi_endproc"),
        );
    }
}

pub unsafe fn restore_context(ctx: &Context) -> ! {
    #[cfg(target_feature = "d")]
    unsafe {
        core::arch::asm!(
            code!(restore_fp),
            code!(restore_gp),
            "
            ld.d $a0, $a0, 8*4
            ret
            ",
            in("$a0") ctx,
            options(noreturn)
        );
    }
    #[cfg(not(target_feature = "d"))]
    unsafe {
        core::arch::asm!(
            code!(restore_gp),
            "
            ld.d $a0, $a0, 8*4
            ret
            ",
            in("$a0") ctx,
            options(noreturn)
        );
    }
}
