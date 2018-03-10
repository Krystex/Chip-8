extern crate rand;

pub mod instruction;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use instruction::Instruction;

/// Describes the Chip-8 Display
#[derive(Copy, Clone)]
pub struct Display {
	arr: [[bool; 64]; 32],
}
impl Display {
	pub const WIDTH: usize = 64;
	pub const HEIGHT: usize = 32;

	/// Creates a new instance of Display
	pub fn new() -> Display {
		Display {
			arr: [[false; Self::WIDTH]; Self::HEIGHT]
		}
	}
	pub fn set(&mut self, x: usize, y: usize, val: bool) {
		self.arr[x][y] = val;
	}

	/// Returns *true* if a collision is happening
	pub fn xor(&mut self, x: usize, y: usize, val: bool) -> bool {
		let previous = self.arr[x][y];
		self.arr[x][y] ^= val;
		previous == true && self.arr[x][y] == false
	}

	/// Calling function *func* for every pixel
	pub fn iterate<F: Fn(usize, usize, bool)>(&self, func: F) {
		for (x, line) in self.arr.iter().enumerate() {
			for (y, val) in line.iter().enumerate() {
				func(x, y, *val);
			}
		}
	}
	/// Switch all pixels off
	pub fn clear_screen(&mut self) {
		self.arr = [[false; Self::WIDTH]; Self::HEIGHT];
	}
}

impl std::fmt::Debug for Display {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		for line in self.arr.iter() {
			for el in line.iter().map(|x| if *x {"X"} else {"_"}) {
				write!(f, "{}", el)?;
			}
			write!(f, "\n")?;
		}
		Ok(())
    }
}


/// Describes the keyboard
#[derive(Debug, Copy, Clone)]
pub struct Keyboard {
	keys: [bool; 16],
}
impl Keyboard {
	pub fn new() -> Keyboard {
		Keyboard {
			keys: [false; 16],
		}
	}
	pub fn down(&mut self, key: u8) {
		self.keys[key as usize] = true;
	}
	pub fn up(&mut self, key: u8) {
		self.keys[key as usize] = false;
	}
	pub fn is_pressed(&self, key: u8) -> bool {
		self.keys[key as usize]
	}
}

/// Get all bit values to a unsigned 8bit value
fn bits(val: u8) -> [bool; 8] {
    let mut result = [false; 8];
    for i in 0..8 {
        let mask = 0b10000000 >> i;
        let bit = (val & mask) >> (8 - i - 1);
        result[i] = if bit == 1 { true } else { false };
    }
    result
}


/// The Emulator System
#[derive(Copy, Clone)]
pub struct System {
	/// General Purpose Registers
	pub regs: [u8; 16],
	/// Special Register **I**
	pub i: u16,
	/// Program Counter (**PC**)
	pub pc: u16,
	/// Stack Pointer (**SP**)
	pub sp: u8,
	/// Delay Timer Register (**DT**)
	pub dt: u8,
	/// Sound Timer Register (**ST**)
	pub st: u8,
	/// Stack
	pub stack: [u16; 16],
	/// Memory
	pub mem: [u8; 4_096],
	/// Display
	pub display: Display,
	/// Keyboard
	pub keyboard: Keyboard,
}

macro_rules! store_sprites {
	($arr:ident => $( $i:expr),* ) => {
		let mut counter = 0;
		$(
			counter += 1;
			$arr[counter - 1] = $i;
		)*
	}
}

impl System {
	/// Creates a new instance with empty memory
	pub fn new() -> System {
		let mut mem = [0u8; 4_096];
		store_sprites! {
			mem =>
			// '0'
			0xF0, 0x90, 0x90, 0x90, 0xF0,
			// '1'
		 	0x20, 0x60, 0x20, 0x20, 0x70,
			// '2'
			0xF0, 0x10, 0xF0, 0x80, 0xF0,
			// '3'
			0xF0, 0x10, 0xF0, 0x10, 0xF0,
			// '4'
			0x90, 0x90, 0xF0, 0x10, 0x10,
			// '5'
			0xF0, 0x80, 0xF0, 0x10, 0xF0,
			// '6'
			0xF0, 0x80, 0xF0, 0x90 ,0xF0,
			// '7'
			0xF0, 0x10, 0x20, 0x40 ,0x40,
			// '8'
			0xF0, 0x90, 0xF0, 0x90 ,0xF0,
			// '9'
			0xF0, 0x90, 0xF0, 0x10 ,0xF0,
			// 'A'
			0xF0, 0x90, 0xF0, 0x90 ,0x90,
			// 'B'
			0xE0, 0x90, 0xE0, 0x90 ,0xE0,
			// 'C'
			0xF0, 0x80, 0x80, 0x80 ,0xF0,
			// 'D'
			0xE0, 0x90, 0x90, 0x90 ,0xE0,
			// 'E'
			0xF0, 0x80, 0xF0, 0x80 ,0xF0,
			// 'F'
			0xF0, 0x80, 0xF0, 0x80, 0x80
		}
		System {
			regs: [0u8; 16],
			i: 0,
			pc: 0,
			sp: 0,
			dt: 0,
			st: 0,
			stack: [0u16; 16],
			mem: mem,
			display: Display::new(),
			keyboard: Keyboard::new(),
		}
	}
	/// Get a reference to a specific (general purpose) register
	pub fn reg(&mut self, id: u8) -> &mut u8 {
		self.regs.get_mut(id as usize).unwrap()
	}

	/// Read a Chip-8 program and put its content into the emulator's memory
	pub fn fetch_file<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
		let file = File::open(path)?;
		for (i, byte) in file.bytes().filter_map(|x| x.ok()).enumerate() {
			// println!("0x{:X}: {:X}", 0x200 + i, byte);
			self.mem[0x200 + i] = byte;
		}
		Ok(())
	}
	/// Increment PC
	pub fn inc_pc(&mut self) {
		self.pc += 2;
	}

	/// Increment SP
	pub fn inc_sp(&mut self) {
		self.sp += 1;
	}

	/// Fetch the next Instruction
	pub fn fetch_instr(&mut self) -> Option<Instruction> {
		let opcode =
			((self.mem[self.pc as usize + 0] as u16) << 8) +
			 self.mem[self.pc as usize + 1] as u16;
		self.inc_pc();
		if self.dt != 0 {
			self.dt -= 1;
		}
		if self.st != 0 {
			self.st -= 1;
		}
		print!("{}[2J", 27 as char);
		std::thread::sleep(std::time::Duration::from_millis(15));
		println!("{:?}", self.display);
		Instruction::parse(opcode)
	}

	/// Run a instruction
	pub fn apply(&mut self, instruction: Instruction) {
		use instruction::Instruction::*;
		// println!("{:?}", instruction);
		match instruction {
			Ld(x, kk) => {
				self.regs[x as usize] = kk;
			}
			LdI(nnn) => {
				self.i = nnn;
			}
			Drw(x, y, length) => {
				let from = self.i as usize;
				let to   = from + length as usize;
				let sprite = self.mem[from .. to].as_ref();
				for (column, byte) in sprite.iter().enumerate() {
					for (row, bit) in bits(*byte).iter().enumerate() {
						let _x = x as usize + row;
						let _y = y as usize + column;
						let collision = self.display.xor(_x, _y, *bit);
						self.regs[0xf] = collision as u8;
					}
				}
			}
			Call(nnn) => {
				self.stack[self.sp as usize] = self.pc;
				self.inc_sp();
				self.pc = nnn;
			}
			LdBCD(x) => {
				// Generate BCD representation
				let val = self.regs[x as usize];
				let a = (val / 100) % 10;
				let b = (val / 10 ) % 10;
				let c = (val / 1  ) % 10;
				// Store it at address I
				self.mem[self.i as usize + 0] = a;
				self.mem[self.i as usize + 1] = b;
				self.mem[self.i as usize + 2] = c;
			}
			LdReadV0(x) => {
				for (i, val) in self.mem.iter().skip(self.i as usize).take(x as usize + 1).enumerate() {
					self.regs[i] = *val;
				}
			}
			LdSprite(x) => {
				self.i = x as u16 * 5;
			}
			AddReg(x, y) => {
				self.regs[x as usize] = self.regs[x as usize].wrapping_add(y);
			}
			Ret => {
				if self.sp == 0 {
					return;
				}
				self.pc = self.stack[self.sp as usize - 1];
				self.sp -= 1;
			}
			LdDelayTimerReg(x) => {
				self.dt = self.regs[x as usize];
			}
			LdDelayTimerValue(x) => {
				self.regs[x as usize] = self.dt;
			}
			SeReg(x, y) => {
				if self.regs[x as usize] == self.regs[y as usize] {
					self.inc_pc();
				}
			}
			SneReg(x, y) => {
				if self.regs[x as usize] != self.regs[y as usize] {
					self.inc_pc();
				}
			}
			Rnd(x, byte) => {
				let rand = rand::random::<u8>();
				self.regs[x as usize] = rand & byte;
			}
			Skp(x) => {
				if self.keyboard.is_pressed(x) {
					self.inc_pc();
				}
			}
			Sknp(x) => {
				if !self.keyboard.is_pressed(x) {
					self.inc_pc();
				}
			}
			And(x, y) => {
				self.regs[x as usize] &= self.regs[y as usize];
			}
			AddCarry(x, y) => {
				let (val, overflowing) = self.regs[x as usize].overflowing_add(self.regs[y as usize]);
				self.regs[x as usize] = val;
				if overflowing {
					self.regs[0xf] = 1;
				} else {
					self.regs[0xf] = 0;
				}
			}
			Se(x, byte) => {
				if self.regs[x as usize] == byte {
					self.inc_pc();
				}
			}
			Sne(x, byte) => {
				if self.regs[x as usize] != byte {
					self.inc_pc();
				}
			}
			Jp(byte) => {
				self.pc = byte;
			}
			LdReg(x, y) => {
				*self.reg(x) = *self.reg(y);
			}
			Sub(x, y) => {
				let (val, overflowing) = (*self.reg(x)).overflowing_sub(*self.reg(y));
				*self.reg(x) = val;
				*self.reg(0xf) = if overflowing { 1 } else { 0 }
			}
			LdSoundTimer(x) => {
				self.st = *self.reg(x);
			}
			AddI(x) => {
				self.i += *self.reg(x) as u16;
			}
			Cls => {
				self.display.clear_screen();
			}
			LdStoreV0(x) => {
				for i in 0..x+1 {
					self.mem[self.i as usize + i as usize] = *self.reg(i);
				}
			}
			_ => (println!("{:?}", instruction))
		}
	}

	/// Run the program from memory
	pub fn run(&mut self) {
		// Set PC to start
		self.pc = 0x200;
		while let Some(ins) = self.fetch_instr() {
			self.apply(ins);
		}
	}
}

fn main() {
	let mut sys = System::new();
	sys.fetch_file("./games/PONG").unwrap();
	sys.run();
	//println!("{:?}", sys.fetch_instr());


}
