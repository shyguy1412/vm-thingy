use std::io::{self, PipeReader, PipeWriter, Read, Write};

#[derive(Debug)]
#[allow(dead_code)]
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
const MIN_STACK_SIZE: usize = 1 << 8;

type Registers = [u16; REGISTER_COUNT as usize];
type Stack = [u16];
type RAM = [u8; RAM_SIZE];

struct Memory<'a> {
    registers: &'a mut Registers,
    stack: &'a mut Stack,
    ram: &'a mut RAM,
}

// #[derive(Debug)]
#[allow(unused)]
pub struct State {
    bin: Box<[u8]>,

    program_ptr: u16,
    registers: Registers,
    stack: Box<Stack>,
    ram: RAM,

    stdout: io::PipeWriter,
    stdin: io::PipeReader,
}

impl State {
    pub fn init_with(bin: &[u8]) -> (Self, (PipeReader, PipeWriter)) {
        let mut ram = [0; RAM_SIZE];

        for i in 0..bin.len() {
            ram[i] = bin[i]
        }

        let (stdout_reader, stdout) = io::pipe().expect("Should be able to create pipe");
        let (stdin, stdin_writer) = io::pipe().expect("Should be able to create pipe");

        (
            Self {
                program_ptr: 0,
                registers: [0; REGISTER_COUNT as usize],
                bin: boxed_copy(bin),
                stack: boxed_slice(MIN_STACK_SIZE),
                ram,
                stdout,
                stdin,
            },
            (stdout_reader, stdin_writer),
        )
    }

    #[allow(unused)]
    pub fn reset(mut self) -> Self {
        self.program_ptr = 0;

        for i in 0..self.registers.len() {
            self.registers[i] = 0;
        }

        self.stack = boxed_slice(MIN_STACK_SIZE);

        for i in 0..self.ram.len() {
            self.ram[i] = *self.bin.get(i).unwrap_or(&0);
        }

        self
    }

    pub fn done(&self) -> bool {
        self.program_ptr == REGISTER_1
    }

    pub fn next(&mut self) {
        let program_ptr @ 0..REGISTER_1 = self.program_ptr else {
            return;
        };

        let stack_ptr = self.stack[0];

        if stack_ptr as usize == (self.stack.len() - 2) / 2 {
            self.expand_stack();
        } else if stack_ptr as usize <= (self.stack.len() - 2) / 4
            && stack_ptr as usize > MIN_STACK_SIZE
        {
            self.shrink_stack();
        }

        let mut memory = Memory {
            registers: &mut self.registers,
            stack: &mut self.stack,
            ram: &mut self.ram,
        };

        let result = match memory.ram[program_ptr as usize] {
            0 => op_halt(), //halt
            1 => op_set(program_ptr, &mut memory),
            2 => op_push(program_ptr, &mut memory),
            3 => op_pop(program_ptr, &mut memory),
            4 => op_eq(program_ptr, &mut memory),
            5 => op_gt(program_ptr, &mut memory),
            6 => op_jmp(program_ptr, &mut memory),
            7 => op_jt(program_ptr, &mut memory),
            8 => op_jf(program_ptr, &mut memory),
            9 => op_add(program_ptr, &mut memory),
            10 => op_mult(program_ptr, &mut memory),
            11 => op_mod(program_ptr, &mut memory),
            12 => op_and(program_ptr, &mut memory),
            13 => op_or(program_ptr, &mut memory),
            14 => op_not(program_ptr, &mut memory),
            15 => op_rmem(program_ptr, &mut memory),
            16 => op_wmem(program_ptr, &mut memory),
            17 => op_call(program_ptr, &mut memory),
            18 => op_ret(program_ptr, &mut memory),
            19 => op_out(program_ptr, &mut memory, &mut self.stdout),
            20 => op_in(program_ptr, &mut memory, &mut self.stdin),
            21 => op_noop(program_ptr, &mut memory), // no-op
            v @ _ => panic!("Invalid instruction: {:02X} at {:02X}", v, program_ptr),
        };

        match result {
            Ok(new_pointer) => self.program_ptr = new_pointer,
            Err(err) => panic!("{}", err),
        }
    }

    fn expand_stack(&mut self) {
        resize_boxed_slice(self.stack.len() * 2, &mut self.stack);
    }

    fn shrink_stack(&mut self) {
        resize_boxed_slice(self.stack.len() / 2, &mut self.stack);
    }
}

fn resize_boxed_slice<T: Copy>(new_size: usize, to_resize: &mut Box<[T]>) {
    unsafe {
        let bytes_to_copy = std::cmp::min(new_size, to_resize.len());
        let mut new_slice = boxed_slice(new_size);
        std::ptr::copy_nonoverlapping(to_resize.as_ptr(), new_slice.as_mut_ptr(), bytes_to_copy);
        *to_resize = new_slice;
    }
}

fn boxed_slice<T>(size: usize) -> Box<[T]> {
    unsafe {
        let Ok(layout) = std::alloc::Layout::array::<T>(size) else {
            panic!("Could not create array layout")
        };
        let ptr = std::alloc::alloc(layout) as *mut T;
        std::ptr::write_bytes(ptr, 0, size);
        let slice: *mut [T] = std::ptr::slice_from_raw_parts_mut(ptr, size);
        Box::from_raw(slice)
    }
}

fn boxed_copy<T: Copy>(to_copy: &[T]) -> Box<[T]> {
    unsafe {
        let layout = std::alloc::Layout::for_value(to_copy);
        let ptr = std::alloc::alloc(layout) as *mut T;
        std::ptr::copy_nonoverlapping(to_copy.as_ptr(), ptr, to_copy.len());
        let slice: *mut [T] = std::ptr::slice_from_raw_parts_mut(ptr, to_copy.len());
        Box::from_raw(slice)
    }
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

    let [stack_ptr, stack @ ..] = memory.stack else {
        unreachable!()
    };

    stack[*stack_ptr as usize] = a;
    *stack_ptr += 1;

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
    let new_ptr = match read_uint15(ptr + 2, memory)? {
        1..=u16::MAX => read_uint15_address(ptr + 4, memory)?,
        0 => ptr + 6,
    };

    Ok(new_ptr)
}

//   8 a b
//   if <a> is zero, jump to <b>
fn op_jf(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
    let new_ptr = match read_uint15(ptr + 2, memory)? {
        0 => read_uint15_address(ptr + 4, memory)?,
        1..=u16::MAX => ptr + 6,
    };

    Ok(new_ptr)
}

macro_rules! operator_operation {
    ($($ident:ident with ($($operand:ident),*) is ($($exp:tt)*))*) => ($(
        fn $ident(ptr: u16, memory: &mut Memory) -> Result<u16, Error> {
            let register = read_register(ptr + 2, memory)?;
            let mut offset = 2;

            $(
                offset += 2;
                let $operand = read_uint15(ptr+offset, memory)?;
            )*

            memory.registers[register] = $($exp)*;

            Ok(ptr + offset + 2)
        }
    )*)
}

operator_operation! {
    op_add  with (a, b) is ((a + b) % REGISTER_1)
    op_mult with (a, b) is (a.wrapping_mul(b) % REGISTER_1)
    op_mod  with (a, b) is (a % b)
    op_and  with (a, b) is (a & b)
    op_or   with (a, b) is (a | b)
    op_not  with (a)    is (!a & ADDRESS_SPACE)
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
    let [stack_ptr, stack @ ..] = memory.stack else {
        unreachable!()
    };

    stack[*stack_ptr as usize] = (ptr >> 1) + 2;
    *stack_ptr += 1;

    let addr = read_uint15_address(ptr + 2, &memory)?;
    Ok(addr)
}

//   18
//   remove the top element from the stack and jump to it; empty stack = halt
fn op_ret(_: u16, memory: &mut Memory) -> Result<u16, Error> {
    let [stack_ptr, stack @ ..] = memory.stack else {
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
fn op_out(ptr: u16, memory: &mut Memory, stdout: &mut PipeWriter) -> Result<u16, Error> {
    let char = read_uint15(ptr + 2, memory)? as u8;
    stdout.write(&[char]).map_err(|e| Error::IOError(e))?;
    Ok(ptr + 4)
}

//   20 a
//   read a character from the terminal and write its ascii code to <a>; it can be assumed that once input starts, it will continue until a newline is encountered; this means that you can safely read whole lines from the keyboard instead of having to figure out how to read individual characters
fn op_in(ptr: u16, memory: &mut Memory, stdin: &mut PipeReader) -> Result<u16, Error> {
    let mut buf: [u8; 1] = [0];
    stdin.read(&mut buf).map_err(|e| Error::IOError(e))?;

    let register = read_register(ptr + 2, memory)?;
    memory.registers[register] = u16::from_le_bytes([buf[0], 0]);

    Ok(ptr + 4)
}

fn op_noop(ptr: u16, _: &mut Memory) -> Result<u16, Error> {
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
