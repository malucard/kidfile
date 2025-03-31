use crate::{byte_slice::ByteSlice, image::{Frame, Image}, Certainty, Decoder};

// https://www.fabiensanglard.net/Mykaruga/tools/segaPVRFormat.txt
// https://dreamcast.wiki/Twiddling

pub const ENTRY_PVR: Decoder<Image> = Decoder {
	id: "pvr",
	desc: "Dreamcast image format",
	detect: |file| {
		let file_start = if file.starts_with(b"GBIX") {16} else {0};
		Certainty::certain_if(file.starts_with_at(b"PVRT", file_start) || file.starts_with_at(b"PVPL", file_start))
	},
	decode: |file| {
		let mut file_start = if file.starts_with(b"GBIX") {16} else {0};
		let mut palette_bytes = Default::default();
		if file.starts_with_at(b"PVPL", file_start) {
			let palette_len = file.read_u32(file_start + 4)? as usize;
			palette_bytes = unsafe {Box::new_uninit_slice(palette_len - 8).assume_init()};
			file.read_chunk_exact(&mut palette_bytes, file_start + 16).map_err(|_| "PVPL length field is incorrect")?;
			file_start += palette_len + 8;
		};
		let file_len = file.read_u32(file_start + 4)? as usize;
		let mut buf = unsafe {Box::new_uninit_slice(file_len + 8).assume_init()};
		file.read_chunk_exact(&mut buf, file_start).map_err(|_| "PVRT length field is incorrect")?;
		if !buf.starts_with(b"PVRT") {
			return Err(format!("PVRT header not found, expected at {:#X}", file_start));
		}
		let pixel_fmt = buf.read_u8(8)?;
		let twiddle_type = buf.read_u8(9)?;
		println!("twiddle type {twiddle_type}");
		let width = buf.read_u16(12)? as usize;
		let height = buf.read_u16(14)? as usize;
		let mut frame = match pixel_fmt {
			0 => Frame::from_bgra16(width as u32, height as u32, buf.get(16..16 + width * height * 2).ok_or("not enough pixel data for BGRA5551")?),
			1 => Frame::from_bgr16(width as u32, height as u32, buf.get(16..16 + width * height * 2).ok_or("not enough pixel data for BGR565")?),
			2 => Frame::from_bgra4444(width as u32, height as u32, buf.get(16..16 + width * height * 2).ok_or("not enough pixel data for BGRA4444")?),
			5 => {
				if palette_bytes.is_empty() {
					Frame::from_bgra_clut4(
						width as u32, height as u32,
						buf.get(16..16 + 1024).ok_or("not enough palette data for BGRA clut4")?,
						buf.get(16 + 1024..16 + 1024 + width * height / 2).ok_or("not enough pixel data for BGRA clut4")?
					)
				} else {
					Frame::from_bgra_clut4(
						width as u32, height as u32,
						&palette_bytes,
						buf.get(16..16 + width * height / 2).ok_or("not enough pixel data for BGRA clut4")?
					)
				}
			}
			6 => {
				if palette_bytes.is_empty() {
					Frame::from_bgra_clut8(
						width as u32, height as u32,
						buf.get(16..16 + 1024).ok_or("not enough palette data for BGRA clut8")?,
						buf.get(16 + 1024..16 + 1024 + width * height).ok_or("not enough pixel data for BGRA clut8")?
					)
				} else {
					Frame::from_bgra_clut8(
						width as u32, height as u32,
						&palette_bytes,
						buf.get(16..16 + width * height).ok_or("not enough pixel data for BGRA clut8")?
					)
				}
			}
			_ => return Err(format!("unhandled PVR pixel format {pixel_fmt}"))
		};
		if [1, 2, 5, 6, 7, 8, 13].contains(&twiddle_type) {
			frame = frame.twiddled_dc();
		}
		Ok(Image {frames: Box::new([frame])})
	}
};