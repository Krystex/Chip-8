//! The Chip-8 Instructions
//!
//! Documentation taken from [http://devernay.free.fr/hacks/chip8/C8TECH10.HTM](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
use std::io::Read;
use std::fs::File;
use std::io;
use std::path::Path;


/// A 12bit value
///
/// Note: isn't actually 12bit because of hardware limitations
pub type Addr = u16;

/// A 4bit value
///
/// Note: isn't actually 12bit because of hardware limitations
pub type Nibble = u8;

/// A 8bit value
pub type Byte = u8;


/// A Chip-8 Instruction
///
///
///
/// *x*   = Nibble
///
/// *y*   = Nibble
///
/// *n*   = Nibble
///
/// *nnn* = Addr
///
/// *kk* = Byte
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Instruction {
	/// **SYS addr**. Jump to a machine code routine at _Addr_.
	Sys(Addr),
	/// **CLS**. Clear the display.
	Cls,
	/// **RET**. Return from a subroutine.
	Ret,
	/// Jump to location _Addr_.
	Jp(Addr),
	/// Call subroutine at _Addr_.
	Call(Addr),
	/// Skip next instruction if Register *x* == Byte.
	Se(Nibble, Byte),
	/// Skip next instruction if Register *x* != Byte.
	Sne(Nibble, Byte),
	/// Skip next instruction if V*x* = V*y*.
	SeReg(Nibble, Nibble),
	/// Set Vx = kk.
	Ld(Nibble, Byte),
	/// Set Vx = Vx + kk.
	AddReg(Nibble, Byte),
	/// Set Vx = Vy.
	LdReg(Nibble, Nibble),
	/// Set Vx = Vx OR Vy.
	Or(Nibble, Nibble),
	/// Set Vx = Vx AND Vy.
	And(Nibble, Nibble),
	/// Set Vx = Vx XOR Vy.
	Xor(Nibble, Nibble),
	/// Set Vx = Vx + Vy, set VF = carry.
	AddCarry(Nibble, Nibble),
	/// Set Vx = Vx - Vy, set VF = NOT borrow.
	Sub(Nibble, Nibble),
	/// Set Vx = Vx SHR 1.
	Shr(Nibble, Nibble),
	/// Set Vx = Vy - Vx, set VF = NOT borrow.
	Subn(Nibble, Nibble),
	/// Set Vx = Vx SHL 1.
	Shl(Nibble, Nibble),
	/// Skip next instruction if Vx != Vy.
	SneReg(Nibble, Nibble),
	/// Set I = nnn.
	LdI(Addr),
	/// Jump to location nnn + V0.
	JpV0(Addr),
	/// Set Vx = random byte AND kk.
	Rnd(Nibble, Byte),
	/// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
	Drw(Nibble, Nibble, Nibble),
	/// Skip next instruction if key with the value of Vx is pressed.
	Skp(Nibble),
	/// Skip next instruction if key with the value of Vx is not pressed.
	Sknp(Nibble),
	/// Set Vx = delay timer value.
	LdDelayTimerValue(Nibble),
	/// Wait for a key press, store the value of the key in Vx.
	LdKeypress(Nibble),
	/// Set delay timer = Vx.
	LdDelayTimerReg(Nibble),
	/// Set sound timer = Vx.
	LdSoundTimer(Nibble),
	/// Set I = I + Vx.
	AddI(Nibble),
	/// Set I = location of sprite for digit Vx.
	LdSprite(Nibble),
	/// Store BCD representation of Vx in memory locations I, I+1, and I+2.
	LdBCD(Nibble),
	/// Store registers V0 through Vx in memory starting at location I.
	LdStoreV0(Nibble),
	/// Read registers V0 through Vx from memory starting at location I.
	LdReadV0(Nibble),
}

fn get_x(val: u16) -> Nibble {
	((val & 0x0f00) >> 8) as u8
}
fn get_y(val: u16) -> Nibble {
	((val & 0x00f0) >> 4) as u8
}
fn get_addr(val: u16) -> Addr {
	val & 0x0fff
}
fn get_byte(val: u16) -> Byte {
	(val & 0x00ff) as u8
}
fn get_nibble(val: u16) -> Nibble {
	(val & 0x000f) as u8
}

impl Instruction {
	/// Get the instruction to the value
	pub fn parse(val: u16) -> Option<Instruction> {
		use self::Instruction::*;
		let x = (
			(val & 0xf000) >> 12,
			(val & 0x0f00) >> 8,
			(val & 0x00f0) >> 4,
			(val & 0x000f) >> 0
		);
		// For debugging
		//println!("{:X}", val);
		match x {
			(0x0, 0x0, 0xe, 0x0) => Some(Cls),
			(0x0, 0x0, 0xe, 0xe) => Some(Ret),
			(0x0, _  , _  , _  ) => Some(Sys(get_addr(val))),
			(0x1, _  , _  , _  ) => {
				let masked = get_addr(val);
				Some(Jp(masked))
			},
			(0x2, _  , _  , _  ) => {
				let masked = get_addr(val);
				Some(Call(masked))
			},
			(0x3, _  , _  , _  ) => {
				let register = get_x(val);
				let value = get_byte(val);
				Some(Se(register, value))
			},
			(0x4, _  , _  , _  ) => {
				let register = get_x(val);
				let value = get_byte(val);
				Some(Sne(register, value))
			},
			(0x5, _  , _  , 0x0) => {
				let x = get_x(val);
				let y = get_y(val);
				Some(SeReg(x, y))
			},
			(0x6, _  , _  , _ ) => {
				let register = get_x(val);
				let value = get_byte(val);
				Some(Ld(register, value))
			},
			(0x7, _  , _  , _ ) => {
				let register = get_x(val);
				let value = get_byte(val);
				Some(AddReg(register, value))
			}
			(0x8, _  , _  , 0x0 ) => {
				let x = get_x(val);
				let y = get_y(val);
				Some(LdReg(x, y))
			},
			(0x8, _  , _  , 0x1 ) => {
				let x = get_x(val);
				let y = get_y(val);
				Some(Or(x, y))
			},
			(0x8, _  , _  , 0x2 ) => {
				let x = get_x(val);
				let y = get_y(val);
				Some(And(x, y))
			},
			(0x8, _  , _  , 0x3 ) => {
				let x = get_x(val);
				let y = get_y(val);
				Some(Xor(x, y))
			},
			(0x8, _  , _  , 0x4 ) => {
				let x = get_x(val);
				let y = get_y(val);
				Some(AddCarry(x, y))
			},
			(0x8, _  , _  , 0x5 ) => {
				let x = get_x(val);
				let y = get_y(val);
				Some(Sub(x, y))
			},
			(0x8, _  , _  , 0x6 ) => {
				let x = get_x(val);
				let y = get_y(val);
				Some(Shr(x, y))
			},
			(0x8, _  , _  , 0x7 ) => {
				let x = get_x(val);
				let y = get_y(val);
				Some(Subn(x, y))
			},
			(0x8, _  , _  , 0xe ) => {
				let x = get_x(val);
				let y = get_y(val);
				Some(Shl(x, y))
			},
			(0x9, _  , _  , 0x0 ) => {
				let x = get_x(val);
				let y = get_y(val);
				Some(SneReg(x, y))
			},
			(0xa, _  , _  , _ ) => {
				let addr = get_addr(val);
				Some(LdI(addr))
			},
			(0xb, _  , _  , _ ) => {
				let addr = get_addr(val);
				Some(JpV0(addr))
			},
			(0xc, _  , _  , _ ) => {
				let x = get_x(val);
				let y = get_byte(val);
				Some(Rnd(x, y))
			},
			(0xd, _  , _  , _ ) => {
				let x = get_x(val);
				let y = get_y(val);
				let n = get_nibble(val);
				Some(Drw(x, y, n))
			},
			(0xe, _ , 0x9, 0xe) => {
				let x = get_x(val);
				Some(Skp(x))
			},
			(0xe, _ , 0xa, 0x1) => {
				let x = get_x(val);
				Some(Sknp(x))
			},
			(0xf, _ , 0x0, 0x7) => {
				let x = get_x(val);
				Some(LdDelayTimerValue(x))
			},
			(0xf, _ , 0x0, 0xa) => {
				let x = get_x(val);
				Some(LdKeypress(x))
			},
			(0xf, _ , 0x1, 0x5) => {
				let x = get_x(val);
				Some(LdDelayTimerReg(x))
			},
			(0xf, _ , 0x1, 0x8) => {
				let x = get_x(val);
				Some(LdSoundTimer(x))
			},
			(0xf, _ , 0x1, 0xe) => {
				let x = get_x(val);
				Some(AddI(x))
			},
			(0xf, _ , 0x2, 0x9) => {
				let x = get_x(val);
				Some(LdSprite(x))
			},
			(0xf, _ , 0x3, 0x3) => {
				let x = get_x(val);
				Some(LdBCD(x))
			},
			(0xf, _ , 0x5, 0x5) => {
				let x = get_x(val);
				Some(LdStoreV0(x))
			},
			(0xf, _ , 0x6, 0x5) => {
				let x = get_x(val);
				Some(LdReadV0(x))
			},
			_ => {
				println!("Not implemented: {:?}", x);
				None
			}
		}
	}
}

pub struct InstructionIterator<R> {
	reader: R,
}
impl<R> InstructionIterator<R> {
	fn new(reader: R) -> InstructionIterator<R> where R: Read{
		InstructionIterator {
			reader: reader,
		}
	}
}
pub fn from_file<P: AsRef<Path>>(file: P) -> io::Result<InstructionIterator<File>> {
	let file = File::open(file).unwrap();
	Ok(InstructionIterator::new(file))
}
impl<T: Read> Iterator for InstructionIterator<T> {
	type Item = Instruction;
	fn next(&mut self) -> Option<Self::Item> {
		let mut data = [0u8; 2];
		if self.reader.read_exact(&mut data).is_err() {
			return None
		}
		let ins: u16 = ((data[0] as u16) << 8) + data[1] as u16;

		Instruction::parse(ins)
	}
}

#[allow(unused_macros)]
macro_rules! test_instr {
	($x:expr, $y:expr) => {
		let test = Instruction::parse($x).unwrap();
		assert_eq!(test, $y);
	}
}

#[test]
fn test_instructions() {
	use self::Instruction::*;

	test_instr!(0x00e0, Cls);
	test_instr!(0x00ee, Ret);
	test_instr!(0x1111, Jp(0x0111));
	test_instr!(0x2111, Call(0x0111));
	test_instr!(0x3111, Se(0x1, 0x11));
	test_instr!(0x4111, Sne(0x1, 0x11));
	test_instr!(0x5110, SeReg(0x1, 0x1));
	test_instr!(0x6111, Ld(0x1, 0x11));
	test_instr!(0x7111, AddReg(0x1, 0x11));
	test_instr!(0x8110, LdReg(0x1, 0x1));
	test_instr!(0x8111, Or(0x1, 0x1));
	test_instr!(0x8112, And(0x1, 0x1));
	test_instr!(0x8113, Xor(0x1, 0x1));
	test_instr!(0x8114, AddCarry(0x1, 0x1));
	test_instr!(0x8115, Sub(0x1, 0x1));
	test_instr!(0x8116, Shr(0x1, 0x1));
	test_instr!(0x8117, Subn(0x1, 0x1));
	test_instr!(0x811e, Shl(0x1, 0x1));
	test_instr!(0x9110, SneReg(0x1, 0x1));
	test_instr!(0xa000, LdI(0x000));
	test_instr!(0xb000, JpV0(0x000));
	test_instr!(0xc111, Rnd(0x1, 0x11));
	test_instr!(0xd123, Drw(0x1, 0x2, 0x3));
	test_instr!(0xe19e, Skp(0x1));
	test_instr!(0xe1a1, Sknp(0x1));
	test_instr!(0xf107, LdDelayTimerValue(0x1));
	test_instr!(0xf10a, LdKeypress(0x1));
	test_instr!(0xf115, LdDelayTimerReg(0x1));
	test_instr!(0xf118, LdSoundTimer(0x1));
	test_instr!(0xf11e, AddI(0x1));
	test_instr!(0xf129, LdSprite(0x1));
	test_instr!(0xf133, LdBCD(0x1));
	test_instr!(0xf155, LdStoreV0(0x1));
	test_instr!(0xf165, LdReadV0(0x1));
}
