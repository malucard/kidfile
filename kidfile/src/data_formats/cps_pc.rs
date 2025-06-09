use crate::{byte_slice::ByteSlice, file_data::FileData, Certainty, Decoder};

pub const ENTRY_CPS_PC: Decoder<Box<[u8]>> = Decoder {
	id: "cps_pc",
	desc: "Old KID PC port obfuscated compression format",
	detect: |buf| Certainty::certain_if(buf.starts_with(b"CPS\0")),
	decode: |data| {
		let buf = data.read();
		let packed_size = buf.read_u32(4)? as usize;
		let compression_type = buf.read_u16(10)?;
		let unpacked_size = buf.read_u32(12)? as usize;

		// deobfuscate
		let mut deobfuscated = Vec::with_capacity(packed_size - 4);
		let key_off = buf.read_u32(packed_size - 4)?.wrapping_sub(0x7534682);
		let mut key = buf.read_u32(key_off as usize)?.wrapping_add(key_off).wrapping_add(0x3786425);
		for pos in 4..16 {
			deobfuscated.push(buf[pos]);
		}
		for pos in (16..packed_size).step_by(4) {
			if pos == packed_size - 4 {
				deobfuscated.push(0);
				break;
			}
			let mut word = buf.get_u32_at(pos).unwrap();
			if pos != key_off as usize && key_off != 0 {
				word = word.wrapping_sub(key.wrapping_add(packed_size as u32));
			}
			for b in word.to_le_bytes() {
				deobfuscated.push(b);
			}
			key = 1103515245u32.wrapping_mul(key).wrapping_add(39686)
		}

		// decompress (maybe)
		if compression_type & 1 != 0 {
			let mut out = Vec::with_capacity(unpacked_size);
			let mut in_pos = 16;
			while in_pos < deobfuscated.len() && out.len() < unpacked_size {
				let ctl = deobfuscated[in_pos] as usize;
				in_pos += 1;
				if ctl & 0x80 != 0 {
					if ctl & 0x40 != 0 {
						let mut count = (ctl & 0x1F) + 2;
						if ctl & 0x20 != 0 {
							count += (deobfuscated[in_pos] as usize) << 5;
							in_pos += 1;
						}
						let remaining_out = unpacked_size - out.len();
						if count > remaining_out {
							count = remaining_out;
						}
						let value = deobfuscated[in_pos];
						in_pos += 1;
						for _ in 0..count {
							out.push(value);
						}
					} else {
						let offset = ((ctl & 3) << 8) + deobfuscated[in_pos] as usize + 1;
						in_pos += 1;
						let count = ((ctl >> 2) & 0xF) + 2;
						let origin = out.len() - offset;
						for i in 0..count {
							out.push(out[origin + i]);
						}
					}
				} else if ctl & 0x40 != 0 {
					let mut size = (ctl & 0x3F) + 2;
					let remaining_in = deobfuscated.len() - in_pos;
					if size > remaining_in {
						size = remaining_in;
					}
					let remaining_out = unpacked_size - out.len();
					if size > remaining_out {
						size = remaining_out;
					}
					let repetitions = deobfuscated[in_pos] as usize + 1;
					in_pos += 1;
					for _ in 0..repetitions {
						if out.len() + size > unpacked_size {
							let len = unpacked_size - out.len();
							for _ in 0..len {
								out.push(deobfuscated[in_pos]);
								in_pos += 1;
							}
							break;
						}
						for i in 0..size {
							out.push(deobfuscated[in_pos + i]);
						}
					}
					in_pos += size;
				} else {
					let mut count = (ctl & 0x1F) + 1;
					if ctl & 0x20 != 0 {
						count += (deobfuscated[in_pos] as usize) << 5;
						in_pos += 1;
					}
					let remaining_in = deobfuscated.len() - in_pos;
					if count > remaining_in {
						count = remaining_in;
					}
					let remaining_out = unpacked_size - out.len();
					if count > remaining_out {
						count = remaining_out;
					}
					for _ in 0..count {
						out.push(deobfuscated[in_pos]);
						in_pos += 1;
					}
				}
			}
			if out.len() == unpacked_size {
				Ok(out.into_boxed_slice())
			} else {
				Err(format!("wrong unpacked size after lnd decompression, expected {unpacked_size}, got {}", out.len()))
			}
		} else if compression_type & 2 != 0 {
			Err("compression type not supported".into())
		} else {
			if deobfuscated.len() >= unpacked_size {
				// for some reason, sometimes the result is a little too big, and yet we're not allowed to trim it
				Ok(deobfuscated.into_boxed_slice())
			} else {
				Err(format!("wrong unpacked size after deobfuscation, expected {unpacked_size}, got {}", deobfuscated.len()))
			}
		}
	}
};
