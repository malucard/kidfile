use crate::{file_data::FileData, Certainty, Decoder};

// based on Never7 PS2 decompilation

pub const ENTRY_CPS: Decoder<Box<[u8]>> = Decoder {
	id: "cps",
	desc: "KID compression format",
	detect: |buf| Certainty::certain_if(buf.starts_with_at(b"ogdt", 4) || buf.starts_with_at(b"TIM2", 4)),
	decode
};

fn decode(data: &mut FileData) -> Result<Box<[u8]>, String> {
	let expected_size = data.get_u32_at_be(0).unwrap() as usize >> 8;

	let buf = &data.read()[3..];
	let mut in_cursor = 0;
	let mut out = vec![0u8; expected_size];
	let mut out_cursor = 0;
	while in_cursor < buf.len() {
		let cur_byte = buf[in_cursor] as usize;
		if cur_byte & 0x80 != 0 { // backreference
			if in_cursor + 1 >= buf.len() {
				return Err("incomplete backreference".into());
			}
			let backref_offset = 1 + ((cur_byte & 3) << 8 | buf[in_cursor + 1] as usize);
			if backref_offset > out_cursor {
				return Err("invalid backreference offset".into());
			}
			let mut backref_addr = out_cursor - backref_offset;
			let backref_len = ((cur_byte & 0x7C) >> 2) + 3;
			in_cursor += 2;
			if backref_addr + backref_len > expected_size {
				return Err("invalid backreference length".into());
			}
			if out_cursor + backref_len > expected_size {
				//break;
				return Err(format!("expected {expected_size} bytes when decompressing, got more from backreference"));
			}
			for _ in 0..backref_len {
				out[out_cursor] = out[backref_addr];
				backref_addr += 1;
				out_cursor += 1;
			}
		} else { // raw chunk
			let mut chunk_len = cur_byte + 1;
			in_cursor += 1;
			if in_cursor + chunk_len > buf.len() {
				//return Err(format!("raw chunk went out of bounds"));
				chunk_len = buf.len() - in_cursor;
			}
			if out_cursor + chunk_len > expected_size {
				if cur_byte == 0 {
					break;
				} else {
					return Err(format!("expected {expected_size} bytes when decompressing, got more from raw chunk"));
				}
			}
			for _ in 0..chunk_len {
				out[out_cursor] = buf[in_cursor];
				in_cursor += 1;
				out_cursor += 1;
			}
		}
	}
	if out_cursor == expected_size {
		Ok(out.into())
	} else {
		Err(format!("expected {expected_size} bytes when decompressing, got only {out_cursor}"))
	}
}
