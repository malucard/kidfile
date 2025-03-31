use crate::{byte_iter::ByteIter, file_data::FileData, Certainty, Decoder};

// thanks to BoilingTeapot for reverse engineering the compression

pub const ENTRY_LZSS_DC: Decoder<Box<[u8]>> = Decoder {
	id: "lzss-dc",
	desc: "LZSS variant used in N7 DC",
	detect: |buf| Certainty::possible_if(decode_header(buf).is_some()),
	decode
};

fn decode_header(data: &mut FileData) -> Option<usize> {
	let size = data.get_u32_at_be(0)?;
	if size > 32 && size < 32 * 1024 * 1024 && data.len() > 32 {
		Some(size as usize)
	} else {
		None
	}
}

fn decode(data: &mut FileData) -> Result<Box<[u8]>, String> {
	if let Some(expected_size) = decode_header(data) {
		match decompress_lzss_dc(&data.read()[4..], expected_size) {
			Ok(decompressed) => Ok(decompressed),
			Err(Some(actual_size)) => Err(format!("expected {expected_size} bytes when decompressing, got only {actual_size}")),
			Err(None) => Err(format!("error while decompressing"))
		}
	} else {
		Err(String::new())
	}
}

fn decompress_lzss_dc(inp: &[u8], expected_size: usize) -> Result<Box<[u8]>, Option<usize>> {
	let mut out = Vec::with_capacity(expected_size);
	let mut src = inp.iter().cloned();
	while out.len() < expected_size {
		let chunk_size = src.next_u16_be().ok_or(out.len())? as usize;
		let mut chunk = src.by_ref().take(chunk_size);
		let chunk_out_start = out.len();
		// the flags byte determines whether the next 8 tokens are literals or references
		while let Some(flags) = chunk.next() {
			for i in 0..8 {
				if flags & (1 << i) == 0 { // literal token
					if let Some(byte) = chunk.next() {
						if out.len() >= expected_size {
							return Err(None);
						}
						out.push(byte);
					} else {
						break;
					}
				} else { // reference token
					let ref_value = chunk.next_u16_be().ok_or(None)? as usize;
					let ref_off = (ref_value >> 5) + 1;
					let ref_len = (ref_value & 0b11111) + 3;
					if ref_off >= out.len() {
						return Err(None);
					}
					let start = out.len() - ref_off;
					if out.len() + ref_len > expected_size {
						return Err(None);
					}
					for i in start..start + ref_len {
						out.push(*out.get(i.max(chunk_out_start)).ok_or(None)?);
					}
				}
			}
		}
	}
	if out.len() == expected_size {
		Ok(out.into_boxed_slice())
	} else {
		Err(Some(out.len()))
	}
}