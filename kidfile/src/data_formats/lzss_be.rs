use crate::{byte_iter::ByteIter, file_data::FileData, Certainty, Decoder};

// thanks to BoilingTeapot for reverse engineering the compression

pub const ENTRY_LZSS_BE: Decoder<Box<[u8]>> = Decoder {
	id: "lzss-be",
	desc: "LZSS-like used in N7 DC and 12R PS2",
	detect: |buf| Certainty::certain_if(decode_header(buf).is_some()),
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
		match decompress_lzss_be(&data.read()[4..], expected_size) {
			Ok(decompressed) => Ok(decompressed),
			Err(Some(actual_size)) => Err(format!("expected {expected_size} bytes when decompressing, got {actual_size}")),
			Err(None) => Err(format!("error while decompressing"))
		}
	} else {
		Err(String::new())
	}
}

fn decompress_lzss_be(inp: &[u8], expected_size: usize) -> Result<Box<[u8]>, Option<usize>> {
	let mut out = Vec::with_capacity(expected_size);
	let mut src = inp.iter().cloned();
	while out.len() < expected_size {
		let chunk_size = src.next_u16_be().ok_or(Some(out.len()))? as usize;
		let mut chunk = src.by_ref().take(chunk_size);
		let chunk_out_start = out.len();
		// the flags byte determines whether the next 8 tokens are literals or references
		while let Some(flags) = chunk.next() {
			for i in 0..8 {
				if flags & (1 << i) == 0 { // literal token
					if let Some(byte) = chunk.next() {
						if out.len() >= expected_size {
							break;
						}
						out.push(byte);
					} else {
						break;
					}
				} else if let Some(ref_value) = chunk.next_u16_be() { // reference token
					let ref_off = (ref_value as usize >> 5) + 1;
					let ref_len = (ref_value as usize & 0b11111) + 3;
					let start = out.len() as isize - ref_off as isize;
					if ref_len > expected_size - out.len() {
						return Err(None);
					}
					for i in start..start + ref_len as isize {
						out.push(*out.get(i.max(chunk_out_start as isize) as usize).ok_or(None)?);
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