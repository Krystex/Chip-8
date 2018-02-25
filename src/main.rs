pub mod instruction;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use instruction::Instruction;

#[derive(Copy, Clone)]
pub struct Display {
	arr: [[bool; 64]; 32],
}
impl Display {
	pub const WIDTH: usize = 64;
	pub const HEIGHT: usize = 32;
	pub fn new() -> Display {
		Display {
			arr: [[false; Self::WIDTH]; Self::HEIGHT]
		}
	}
	pub fn set(&mut self, x: usize, y: usize, val: bool) {
		self.arr[x][y] = val;
	}
	pub fn iterate<F: Fn(usize, usize, bool)>(&self, func: F) {
		for (x, line) in self.arr.iter().enumerate() {
			for (y, val) in line.iter().enumerate() {
				func(x, y, *val);
			}
		}
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

/// The Emulator System
#[derive(Copy, Clone)]
pub struct System {
	/// General Purpose Registers
	pub regs: [u8; 16],
	/// Special Register **I**
	pub i: u16,
	/// Special Register **VF**
	pub vf: u8,
	/// Program Counter (**PC**)
	pub pc: u16,
	/// Stack Pointer (**SP**)
	pub sp: u8,
	/// Stack
	pub stack: [u16; 16],
	/// Memory
	pub mem: [u8; 4_096],
	/// Display
	pub display: Display,
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
			vf: 0,
			pc: 0,
			sp: 0,
			stack: [0u16; 16],
			mem: [0u8; 4_096],
			display: Display::new(),
		}
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
			/*Drw(x, y, length) => {
				println!("From 0x{:X} to 0x{:X}", self.i, self.i + length as u16);
				let from = self.i as usize;
				let to   = from + length as usize;
				let sprite = &self.mem[from .. to];
				println!("Sprite: {:?}", sprite);
				for (i, value) in sprite.iter().enumerate() {
					//self.display.set(x as usize + i, y, value);
				}
			}*/
			Call(nnn) => {
				self.inc_sp();
				self.pc = nnn;
			}
			LdBCD(x) => {
				// Generate BCD representation
				let val = self.regs[x as usize];
				let a = (val / 100) % 10;
				let b = (val / 10 ) % 10;
				let c = (val / 1  ) % 10;
				self.mem[self.i as usize + 0] = a;
				self.mem[self.i as usize + 1] = b;
				self.mem[self.i as usize + 2] = c;
			}
			_ => (println!("{:?}", instruction)),
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
