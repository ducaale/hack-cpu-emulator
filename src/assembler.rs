use std::collections::HashMap;
use phf::{Map, phf_map};
use crate::utils::{get_bit, get_bit_slice};

static DEST_SYMBOLS: Map<&'static str, i16> = phf_map! {
    "M" =>  0b001, "D" =>   0b010, "MD" => 0b011, "A" => 0b100, "AM" => 0b101,
    "AD" => 0b110, "AMD" => 0b111
};

static COMP_SYMBOLS: Map<&'static str, i16> = phf_map! {
    "0"   => 0b0101010, "1"   => 0b0111111, "-1"  => 0b0111010, "D"   => 0b0001100,
    "A"   => 0b0110000, "M"   => 0b1110000, "!D"  => 0b0001101, "!A"  => 0b0110001,
    "!M"  => 0b1110001, "-D"  => 0b0001111, "-A"  => 0b0110011, "-M"  => 0b1110011,
    "D+1" => 0b0011111, "A+1" => 0b0110111, "M+1" => 0b1110111, "D-1" => 0b0001110,
    "A-1" => 0b0110010, "M-1" => 0b1110010, "D+A" => 0b0000010, "D+M" => 0b1000010,
    "D-A" => 0b0010011, "D-M" => 0b1010011, "A-D" => 0b0000111, "M-D" => 0b1000111,
    "D&A" => 0b0000000, "D&M" => 0b1000000, "D|A" => 0b0010101, "D|M" => 0b1010101
};

static JUMP_SYMBOLS: Map<&'static str, i16> = phf_map! {
    "JGT" => 0b001, "JEQ" => 0b010, "JGE" => 0b011, "JLT" => 0b100, "JNE" => 0b101,
    "JLE" => 0b110, "JMP" => 0b111
};

static R_DEST_SYMBOLS: [&'static str; 8] = ["", "M", "D", "MD", "A", "AM", "AD", "AMD"];

static R_COMP_SYMBOLS: Map<i16, &'static str> = phf_map! {
    0b0101010i16 => "0",   0b0111111i16 => "1",   0b0111010i16 => "-1",  0b0001100i16 => "D",
    0b0110000i16 => "A",   0b1110000i16 => "M",   0b0001101i16 => "!D",  0b0110001i16 => "!A",
    0b1110001i16 => "!M",  0b0001111i16 => "-D" , 0b0110011i16 => "-A",  0b1110011i16 => "-M",
    0b0011111i16 => "D+1", 0b0110111i16 => "A+1", 0b1110111i16 => "M+1", 0b0001110i16 => "D-1",
    0b0110010i16 => "A-1", 0b1110010i16 => "M-1", 0b0000010i16 => "D+A", 0b1000010i16 => "D+M",
    0b0010011i16 => "D-A", 0b1010011i16 => "D-M", 0b0000111i16 => "A-D", 0b1000111i16 => "M-D",
    0b0000000i16 => "D&A", 0b1000000i16 => "D&M", 0b0010101i16 => "D|A", 0b1010101i16 => "D|M"
};

static R_JUMP_SYMBOLS: [&'static str; 8] = ["", "JGT", "JEQ", "JGE", "JLT", "JNE", "JLE", "JMP"];

#[derive(Debug)]
enum Command<'a> {
    A { address: &'a str, line_number: usize },
    C { dest: Option<&'a str>, comp: &'a str, jump: Option<&'a str>, line_number: usize },
    L { label: &'a str, line_number: usize },
}

fn tokenize(input: &Vec<String>) -> Vec<Command> {
    let mut commands = vec![];

    for (line_number, line) in input.iter().enumerate() {
        let line = {
            let line = line.trim_start();
            let end = line.find('/').unwrap_or(line.len());
            line[0..end].trim_end()
        };

        match line.chars().next() {
            Some('/') | None => continue,
            Some('@') => commands.push(Command::A {
                address: &line[1..line.len()],
                line_number
            }),
            Some('(') => commands.push(Command::L {
                label: &line[1..line.len()-1],
                line_number
            }),
            Some(_) => {
                let eq_sep = line[0..line.len()].find('=');
                let colon_sep = line[0..line.len()].find(';');

                let dest = eq_sep.map(|end| &line[0..end]);
                let comp = {
                    let start = eq_sep.map(|start| start + 1).unwrap_or(0);
                    let end = colon_sep.unwrap_or(line.len());
                    &line[start..end]
                };
                let jump = colon_sep.map(|start| &line[start + 1..line.len()]);
                commands.push(Command::C { dest, comp, jump, line_number })
            }
        };
    }

    commands
}

fn transform(commands: &Vec<Command>, symbol_table: &mut HashMap<String, u32>) -> Vec<i16> {
    let mut current_line = 0;

    for command in commands.iter() {
        match *command {
            Command::L { label, .. } => {
                symbol_table.insert(label.to_owned(), current_line);
            }
            _ => {
                current_line += 1;
            }
        }
    }
    
    let mut memory_address = 15;
    let mut binary_code = vec![];

    for command in commands.iter() {
        match command {
            Command::A { address, .. } => {
                match address.parse::<i16>() {
                    Ok(number) => {
                        binary_code.push(number);
                    }
                    Err(_) => {
                        let number = symbol_table.entry(address.to_string())
                            .or_insert_with(|| {
                                memory_address += 1;
                                memory_address
                            });
                        binary_code.push(*number as i16);
                    }
                };
            }
            Command::C { dest, comp, jump, line_number } => {
                let c_bits = match COMP_SYMBOLS.get(*comp) {
                    Some(v) => *v,
                    None => {
                        let error_message = format!("invalid comp on line {}: {}", line_number + 1, comp);
                        let rcomp: Vec<char> = comp.chars().rev().collect();
                        if rcomp.len() != 3 || rcomp[1] == '-' { panic!(error_message); }
                        let rcomp: String = rcomp.iter().collect();
                        *COMP_SYMBOLS.get(&rcomp as &str).expect(&error_message)
                    }
                };

                let d_bits = dest.map_or(0b000, |dest|
                    *DEST_SYMBOLS.get(dest).expect(&format!(
                        "invalid dest on line {}: {}",
                        line_number + 1,
                        dest
                    ))
                );
                let j_bits = jump.map_or(0b000, |jump|
                    *JUMP_SYMBOLS.get(jump).expect(&format!(
                        "invalid jump on line {}: {}",
                        line_number + 1,
                        jump
                    ))
                );
                let b = (0b111 << 13) + (c_bits << 6) + (d_bits << 3) + (j_bits);
                binary_code.push(b);
            }
            _ => {}
        };
    }

    binary_code
}

fn init_symbol_table() -> HashMap<String, u32> {
    let mut symbol_table = HashMap::new();
    symbol_table.insert(String::from("SP"), 0);
    symbol_table.insert(String::from("LCL"), 1);
    symbol_table.insert(String::from("ARG"), 2);
    symbol_table.insert(String::from("THIS"), 3);
    symbol_table.insert(String::from("THAT"), 4);
    symbol_table.insert(String::from("SCREEN"), 16384);
    symbol_table.insert(String::from("KBD"), 24576);
    (0..16).for_each(|i| {
        symbol_table.insert(format!("R{}", i), i);
    });

    symbol_table
}

pub fn assemble(input: &Vec<String>) -> Vec<i16> {
    let mut symbol_table = init_symbol_table();
    let commands = tokenize(&input);
    let binary_code = transform(&commands, &mut symbol_table);
    binary_code
}

pub fn to_asm(instr: i16) -> String {
    let is_a_instr = !get_bit(instr, 15);
    if is_a_instr {
        format!("@{}", instr)
    }
    else {
        let comp_bits = get_bit_slice(instr, 6, 13);
        let dest_bits = get_bit_slice(instr, 3, 6);
        let jump_bits = get_bit_slice(instr, 0, 3);
        let mut asm = "".to_owned();

        if dest_bits != 0 {
            asm.push_str(&format!("{}=", R_DEST_SYMBOLS[dest_bits as usize]));
        }
        asm.push_str(R_COMP_SYMBOLS.get(&comp_bits).unwrap());
        if jump_bits != 0 {
            asm.push_str(&format!(";{}", R_JUMP_SYMBOLS[jump_bits as usize]));
        }
        
        asm
    }
}