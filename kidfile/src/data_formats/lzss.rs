use crate::{file_data::FileData, Certainty, Decoder};

pub const ENTRY_LZSS: Decoder<Box<[u8]>> = Decoder {
	id: "lzss",
	desc: "The common lzss.c from Haruhiko Okumura",
	detect: |buf| Certainty::possible_if(decode_header(buf).is_some()),
	decode
};

fn decode_header(data: &mut FileData) -> Option<usize> {
	match data.get_u32_at(0) {
		Ok(size) if size > 32 && size < 32 * 1024 * 1024 && data.len() > 32 => Some(size as usize),
		_ => None
	}
}

fn decode(data: &mut FileData) -> Result<Box<[u8]>, String> {
	if let Some(expected_size) = decode_header(data) {
		println!("lzss expecting size {expected_size}, file size is {}", data.len());
		match decompress_lzss(&data.read()[4..], expected_size) {
			Ok(decompressed) => Ok(decompressed),
			Err(Some(actual_size)) => Err(format!("expected {expected_size} bytes when decompressing, got only {actual_size}")),
			Err(None) => Err(format!("expected {expected_size} bytes when decompressing, got more"))
		}
	} else {
		Err(String::new())
	}
}

fn decompress_lzss(inp: &[u8], expected_size: usize) -> Result<Box<[u8]>, Option<usize>> {
	let mut out = Vec::with_capacity(expected_size);
	let mut src = inp.iter();
	let mut flags = 0;
	const N: usize = 4096;
	const F: usize = 18;
	let mut text_buf = [0u8; N + F - 1];
	let mut r = N - F;
	loop {
		flags >>= 1;
		if flags & 0x100 == 0 {
			if let Some(c) = src.next() {
				flags = *c as u32 | 0xFF00; // use higher byte cleverly to count eight
			} else {
				break;
			}
		}
		if flags & 1 != 0 {
			if let Some(c) = src.next() {
				if out.len() >= expected_size {
					return Err(None);
				}
				out.push(*c);
				text_buf[r] = *c;
				r = (r + 1) & N - 1;
			}
		} else {
			if let (Some(mut i), Some(mut j)) = (src.next().cloned(), src.next().cloned()) {
				i |= (j & 0xF0) << 4;
				j = (j & 0x0F) + 2;
				for k in 0..=j {
					let c = text_buf[(i as usize + k as usize) & (N - 1)];
					if out.len() >= expected_size {
						return Err(None);
					}
					out.push(c);
					text_buf[r] = c;
					r = (r + 1) & N - 1;
				}
			} else {
				break;
			}
		}
	}
	if out.len() == expected_size {
		Ok(out.into_boxed_slice())
	} else {
		Err(Some(out.len()))
	}
}