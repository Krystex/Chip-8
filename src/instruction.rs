//! The Chip-8 Instructions
//!
//! Documentation taken from [http://devernay.free.fr/hacks/chip8/C8TECH10.HTM](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)



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
		match x {
			(0x0, 0x0, 0xe, 0x0) => Some(Cls),
			(0x0, 0x0, 0xe, 0xe) => Some(Ret),
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
			_ => unimplemented!()
		}
	}
}


#[test]
fn test_instructions() {
	use self::Instruction::*;

	let cls = Instruction::parse(0x00e0).unwrap();
	assert_eq!(cls, Cls);

	let ret = Instruction::parse(0x00ee).unwrap();
	assert_eq!(ret, Ret);

	let jp = Instruction::parse(0x1111).unwrap();
	assert_eq!(jp, Jp(0x0111));

	let call = Instruction::parse(0x2111).unwrap();
	assert_eq!(call, Call(0x0111));

	let se = Instruction::parse(0x3111).unwrap();
	assert_eq!(se, Se(0x1, 0x11));

	let sne = Instruction::parse(0x4111).unwrap();
	assert_eq!(sne, Sne(0x1, 0x11));

	let se_reg = Instruction::parse(0x5110).unwrap();
	assert_eq!(se_reg, SeReg(0x1, 0x1));

	let ld = Instruction::parse(0x6111).unwrap();
	assert_eq!(ld, Ld(0x1, 0x11));

	let addreg = Instruction::parse(0x7111).unwrap();
	assert_eq!(addreg, AddReg(0x1, 0x11));

	let ldreg = Instruction::parse(0x8110).unwrap();
	assert_eq!(ldreg, LdReg(0x1, 0x1));

	let or = Instruction::parse(0x8111).unwrap();
	assert_eq!(or, Or(0x1, 0x1));

	let and = Instruction::parse(0x8112).unwrap();
	assert_eq!(and, And(0x1, 0x1));

	let xor = Instruction::parse(0x8113).unwrap();
	assert_eq!(xor, Xor(0x1, 0x1));

	let add = Instruction::parse(0x8114).unwrap();
	assert_eq!(add, AddCarry(0x1, 0x1));

	let sub = Instruction::parse(0x8115).unwrap();
	assert_eq!(sub, Sub(0x1, 0x1));

	let shr = Instruction::parse(0x8116).unwrap();
	assert_eq!(shr, Shr(0x1, 0x1));

	let subn = Instruction::parse(0x8117).unwrap();
	assert_eq!(subn, Subn(0x1, 0x1));
}
