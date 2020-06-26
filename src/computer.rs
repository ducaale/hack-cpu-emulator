use crate::utils::{get_bit, get_bit_slice};

pub const KBD_ADDRESS: usize = 24_576;
pub const SCR_ADDRESS: usize = 16_384;

pub struct Computer {
    pub d_register: i16,
    pub a_register: i16,
    pub pc: i16,
    pub rom: [Option<i16>; 1000],
    pub memory: [i16; 24_577]
}

impl Computer {
    pub fn new() -> Computer {
        Computer {
            d_register: 0,
            a_register: 0,
            pc: 0,
            rom: [None; 1000],
            memory: [0; 24_577]
        }
    }

    fn alu(&self, x: i16, y: i16, comp_bits: i16) -> (i16, bool, bool) {
        let mut x = x;
        let mut y = y;
        let mut out;

        let zx = get_bit(comp_bits, 5);
        let nx = get_bit(comp_bits, 4);
        let zy = get_bit(comp_bits, 3);
        let ny = get_bit(comp_bits, 2);
        let f = get_bit(comp_bits, 1);
        let no = get_bit(comp_bits, 0);


        if zx { x = 0; }
        if nx { x = !x; }
        if zy { y = 0; }
        if ny { y = !y; }
        out = if f { x + y } else { x & y };
        if no { out = !out };

        let zr = out == 0;
        let ng = out < 0;

        (out, zr, ng)
    }

    pub fn step(&mut self) {
        let instr = self.rom[self.pc as usize].unwrap_or(0);
        let is_a_instr = !get_bit(instr, 15);
        let a_bit = get_bit(instr, 12);
        let comp_bits = get_bit_slice(instr, 6, 12);
        let dest_bits = get_bit_slice(instr, 3, 6);
        let jump_bits = get_bit_slice(instr, 0, 3);

        if is_a_instr {
            self.a_register = instr;
            self.pc += 1;
        } else {
            let x = self.d_register;
            let y = if !a_bit {
                self.a_register
            } else {
                self.memory[self.a_register as usize]
            };
            let (alu_output, zr, ng) = self.alu(x, y, comp_bits);

            if get_bit(dest_bits, 2) {
                self.a_register = alu_output
            }
            if get_bit(dest_bits, 1) {
                self.d_register = alu_output
            }
            if get_bit(dest_bits, 0) {
                self.memory[self.a_register as usize] = alu_output
            }
 
            let should_jump = match jump_bits {
              0 => false,        // null
              1 => !(zr || ng),  // JGT
              2 => zr,           // JEQ
              3 => !ng,          // JGE
              4 => ng,           // JLT
              5 => !zr,          // JNE
              6 => ng || zr,     // JLE
              _ => true,         // JMP
            };

            self.pc = if should_jump {
                self.a_register
            } else {
                self.pc + 1
            }
        }
    }

}
