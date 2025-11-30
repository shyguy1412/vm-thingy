#![allow(unused_variables)]

use std::io::{self, Read};

#[derive(Debug)]
enum Error {
    InvalidAddress(u16),
    InvalidUint15(u16),
    InvalidRegister(u16),
    EmptyStack,
    IOError(io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidAddress(addr) => write!(f, "Invalid Address: {}", addr),
            Error::InvalidUint15(int) => write!(f, "Invalid Uint15: {}", int),
            Error::InvalidRegister(reg) => write!(f, "Invalid Register: {}", reg),
            Error::EmptyStack => write!(f, "Empty Stack"),
            Error::IOError(_) => write!(f, "IO Error"),
        }
    }
}

const REGISTER_COUNT: u16 = 8;
const WORD_BITS: u8 = 15;

const ADDRESS_SPACE: u16 = !(1 << WORD_BITS);
const RAM_SIZE: usize = 1 << WORD_BITS + 1;
const REGISTER_SPACE: u16 = ADDRESS_SPACE + REGISTER_COUNT;
const REGISTER_1: u16 = ADDRESS_SPACE + 1;
const INVALID_START: u16 = ADDRESS_SPACE + REGISTER_COUNT + 1;
const MIN_STACK_SIZE: usize = 1 << 11;

type Registers = [u16; REGISTER_COUNT as usize];
type Stack<'a> = &'a mut [u16];

struct Memory<'a> {
    pub registers: Registers,
    pub stack: Stack<'a>,
    pub ram: [u8; RAM_SIZE],
}

fn main() {
    let mut binary_pointer = 0u16;

    let mut memory = Memory {
        registers: [0; REGISTER_COUNT as usize],
        //start with 4kb stack. index 0 is the stack pointer
        stack: &mut [0u16; MIN_STACK_SIZE + 1],
        ram: [0; RAM_SIZE],
    };

    const BINARY: &[u8; 60100] = include_bytes!("../challenge.bin");

    for i in 0..BINARY.len() {
        memory.ram[i] = BINARY[i]
    }

    loop {
        if binary_pointer % 2 != 0 {
            panic!("INVALID POINTER: {:02X}", binary_pointer)
        }

        let result = match memory.ram[binary_pointer as usize] {
            0 => op_halt(), //halt
            1 => op_set(binary_pointer, &mut memory),
            2 => op_push(binary_pointer, &mut memory),
            3 => op_pop(binary_pointer, &mut memory),
            4 => op_eq(binary_pointer, &mut memory),
            5 => op_gt(binary_pointer, &mut memory),
            6 => op_jmp(binary_pointer, &mut memory),
            7 => op_jt(binary_pointer, &mut memory),
            8 => op_jf(binary_pointer, &mut memory),
            9 => op_add(binary_pointer, &mut memory),
            10 => op_mult(binary_pointer, &mut memory),
            11 => op_mod(binary_pointer, &mut memory),
            12 => op_and(binary_pointer, &mut memory),
            13 => op_or(binary_pointer, &mut memory),
            14 => op_not(binary_pointer, &mut memory),
            15 => op_rmem(binary_pointer, &mut memory),
            16 => op_wmem(binary_pointer, &mut memory),
            17 => op_call(binary_pointer, &mut memory),
            18 => op_ret(binary_pointer, &mut memory),
            19 => op_out(binary_pointer, &mut memory),
            20 => op_in(binary_pointer, &mut memory),
            21 => op_noop(binary_pointer, &mut memory), // no-op
            v @ _ => panic!("Invalid instruction: {:02X} at {:02X}", v, binary_pointer),
        };

        match result {
            Ok(new_pointer @ REGISTER_1) => break,
            Ok(new_pointer) => binary_pointer = new_pointer,
            Err(err) => panic!("{}", err),
        }
    }

    println!("Terminated");
}

//   halt: 0
//   stop execution and terminate the program
fn op_halt() -> Result<u16, Error> {
    Ok(REGISTER_1)
}

//   1 a b
//   set register <a> to the value of <b>
fn op_set(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let register = read_register(ptr + 2, memory)?;
    let value = read_uint15(ptr + 4, memory)?;
    memory.registers[register] = value;
    Ok(ptr + 6)
}

//   2 a
//   push <a> onto the stack
fn op_push(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let a = read_uint15(ptr + 2, memory)?;

    let Memory {
        registers,
        stack: [stack_ptr, stack @ ..],
        ram,
    } = memory
    else {
        unreachable!()
    };

    stack[*stack_ptr as usize] = a;
    *stack_ptr += 1;

    if *stack_ptr as usize >= stack.len() {
        todo!("Dynamic Stack")
    }

    Ok(ptr + 4)
}

//   3 a
//   remove the top element from the stack and write it into <a>; empty stack = error
fn op_pop(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let register = read_register(ptr + 2, memory)?;

    let Memory {
        registers,
        stack: [stack_ptr, stack @ ..],
        ram: _,
    } = memory
    else {
        unreachable!()
    };

    if *stack_ptr == 0 {
        return Err(Error::EmptyStack);
    }

    *stack_ptr -= 1;
    registers[register] = stack[*stack_ptr as usize];

    if *stack_ptr as usize <= stack.len() / 4 && stack.len() > MIN_STACK_SIZE {
        todo!("Dynamic Stack")
    }

    Ok(ptr + 4)
}

//   4 a b c
//   set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
fn op_eq(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let register = read_register(ptr + 2, memory)?;
    let a = read_uint15(ptr + 4, memory)?;
    let b = read_uint15(ptr + 6, memory)?;

    memory.registers[register] = (a == b) as u16;

    Ok(ptr + 8)
}

//   5 a b c
//   set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
fn op_gt(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let register = read_register(ptr + 2, memory)?;
    let a = read_uint15(ptr + 4, memory)?;
    let b = read_uint15(ptr + 6, memory)?;

    memory.registers[register] = (a > b) as u16;

    Ok(ptr + 8)
}

//   6 a
//   jump to <a>
fn op_jmp(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    Ok(read_uint15_address(ptr + 2, memory)?)
}

//   7 a b
//   if <a> is nonzero, jump to <b>
fn op_jt(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let registers = &memory.registers;
    let new_ptr = match read_uint15(ptr + 2, memory)? {
        v @ 1..=u16::MAX => read_uint15_address(ptr + 4, memory)?,
        v @ 0 => ptr + 6,
    };

    Ok(new_ptr)
}

//   8 a b
//   if <a> is zero, jump to <b>
fn op_jf(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let registers = &memory.registers;
    let new_ptr = match read_uint15(ptr + 2, memory)? {
        v @ 0 => read_uint15_address(ptr + 4, memory)?,
        v @ 1..=u16::MAX => ptr + 6,
    };

    Ok(new_ptr)
}

//   9 a b c
//   assign into <a> the sum of <b> and <c> (modulo 32768)
fn op_add(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let register = read_register(ptr + 2, memory)?;
    let a = read_uint15(ptr + 4, memory)?;
    let b = read_uint15(ptr + 6, memory)?;

    memory.registers[register] = (a + b) % REGISTER_1;

    Ok(ptr + 8)
}

//   10 a b c
//   store into <a> the product of <b> and <c> (modulo 32768)
fn op_mult(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let register = read_register(ptr + 2, memory)?;
    let a = read_uint15(ptr + 4, memory)?;
    let b = read_uint15(ptr + 6, memory)?;

    memory.registers[register] = a.wrapping_mul(b) % REGISTER_1;

    Ok(ptr + 8)
}

//   11 a b c
//   store into <a> the remainder of <b> divided by <c>
fn op_mod(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let register = read_register(ptr + 2, memory)?;
    let a = read_uint15(ptr + 4, memory)?;
    let b = read_uint15(ptr + 6, memory)?;

    memory.registers[register] = a % b;

    Ok(ptr + 8)
}

//   12 a b c
//   stores into <a> the bitwise and of <b> and <c>
fn op_and(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let register = read_register(ptr + 2, memory)?;
    let a = read_uint15(ptr + 4, memory)?;
    let b = read_uint15(ptr + 6, memory)?;

    memory.registers[register] = a & b;

    Ok(ptr + 8)
}

//   13 a b c
//   stores into <a> the bitwise or of <b> and <c>
fn op_or(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let register = read_register(ptr + 2, memory)?;
    let a = read_uint15(ptr + 4, memory)?;
    let b = read_uint15(ptr + 6, memory)?;

    memory.registers[register] = a | b;

    Ok(ptr + 8)
}

//   14 a b
//   stores 15-bit bitwise inverse of <b> in <a>
fn op_not(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let register = read_register(ptr + 2, memory)?;
    let a = read_uint15(ptr + 4, memory)?;

    memory.registers[register] = !a & ADDRESS_SPACE;

    Ok(ptr + 6)
}

//   15 a b
//   read memory at address <b> and write it to <a>
fn op_rmem(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let register = read_register(ptr + 2, memory)?;
    let addr = read_uint15_address(ptr + 4, memory)?;
    let value = read_uint15(addr, memory)?;

    memory.registers[register] = value;

    Ok(ptr + 6)
}

//   16 a b
//   write the value from <b> into memory at address <a>
fn op_wmem(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let addr = read_uint15_address(ptr + 2, &memory)?;
    let [byte1, byte2] = read_uint15(ptr + 4, memory)?.to_le_bytes();

    memory.ram[addr as usize] = byte1;
    memory.ram[addr as usize + 1] = byte2;

    Ok(ptr + 6)
}

//   17 a
//   write the address of the next instruction to the stack and jump to <a>
fn op_call(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let Memory {
        registers,
        stack: [stack_ptr, stack @ ..],
        ram,
    } = memory
    else {
        unreachable!()
    };

    stack[*stack_ptr as usize] = (ptr >> 1) + 2;
    *stack_ptr += 1;

    let addr = read_uint15_address(ptr + 2, &memory)?;
    Ok(addr)
}

//   18
//   remove the top element from the stack and jump to it; empty stack = halt
fn op_ret(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let Memory {
        registers,
        stack: [stack_ptr, stack @ ..],
        ram,
    } = memory
    else {
        unreachable!()
    };

    if *stack_ptr == 0 {
        return Ok(REGISTER_1);
    }

    *stack_ptr -= 1;
    let addr = stack[*stack_ptr as usize] << 1;

    Ok(addr)
}

//   19 a
//   write the character represented by ascii code <a> to the terminal
fn op_out(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let char = read_uint15(ptr + 2, memory)? as u8 as char;
    print!("{char}");
    Ok(ptr + 4)
}

//   20 a
//   read a character from the terminal and write its ascii code to <a>; it can be assumed that once input starts, it will continue until a newline is encountered; this means that you can safely read whole lines from the keyboard instead of having to figure out how to read individual characters
fn op_in(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let mut buf: [u8; 1] = [0];
    io::stdin().read(&mut buf).map_err(|e| Error::IOError(e))?;
    let register = read_register(ptr + 2, memory)?;
    memory.registers[register] = u16::from_le_bytes([buf[0], 0]);

    Ok(ptr + 4)
}

fn op_noop(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    Ok(ptr + 2)
}

fn read_uint15(ptr: u16, memory: &Memory) -> Result<u16, Error> {
    let uint15 = u16::from_le_bytes([memory.ram[ptr as usize], memory.ram[ptr as usize + 1]]);

    match uint15 {
        0..=ADDRESS_SPACE => Ok(uint15),
        REGISTER_1..=REGISTER_SPACE => Ok(memory.registers[(uint15 - REGISTER_1) as usize]),
        INVALID_START..=u16::MAX => Err(Error::InvalidUint15(uint15)),
    }
}

fn read_register(ptr: u16, memory: &Memory) -> Result<usize, Error> {
    let uint15 = u16::from_le_bytes([memory.ram[ptr as usize], memory.ram[ptr as usize + 1]]);

    match uint15 {
        0..=ADDRESS_SPACE => Err(Error::InvalidRegister(uint15)),
        REGISTER_1..=REGISTER_SPACE => Ok((uint15 - REGISTER_1) as usize),
        INVALID_START..=u16::MAX => Err(Error::InvalidUint15(uint15)),
    }
}

fn read_uint15_address(ptr: u16, memory: &Memory) -> Result<u16, Error> {
    let uint15 = u16::from_le_bytes([memory.ram[ptr as usize], memory.ram[ptr as usize + 1]]);

    match uint15 {
        0..=ADDRESS_SPACE => Ok(uint15 << 1),
        REGISTER_1..=REGISTER_SPACE => Ok(memory.registers[(uint15 - REGISTER_1) as usize] << 1),
        INVALID_START..=u16::MAX => Err(Error::InvalidAddress(uint15)),
    }
}
