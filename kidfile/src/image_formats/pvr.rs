use crate::{byte_slice::ByteSlice, image::{Frame, Image}, Certainty, Decoder};

// https://www.fabiensanglard.net/Mykaruga/tools/segaPVRFormat.txt

pub const ENTRY_PVR: Decoder<Image> = Decoder {
	id: "pvr",
	desc: "Dreamcast image format",
	detect: |file| Certainty::certain_if(file.starts_with(b"PVRT") || (file.starts_with(b"GBIX") && file.starts_with_at(b"PVRT", 16))),
	decode: |file| {
		let gbix = file.starts_with(b"GBIX");
		let file_len = file.read_u32(if gbix {20} else {4})? as usize;
		let mut buf = unsafe {Box::new_uninit_slice(file_len + 8).assume_init()};
		file.read_chunk_exact(&mut buf, if gbix {16} else {0}).map_err(|_| "file length field is incorrect")?;
		let pixel_fmt = buf.read_u8(8)?;
		let twiddle_type = buf.read_u8(9)?;
		println!("twiddle type {twiddle_type}");
		let width = buf.read_u16(12)? as usize;
		let height = buf.read_u16(14)? as usize;
		let frame = match pixel_fmt {
			0 => Frame::from_bgra16(width as u32, height as u32, buf.get(16..16 + width * height * 2).ok_or("not enough pixel data")?),
			1 => Frame::from_bgr16(width as u32, height as u32, buf.get(16..16 + width * height * 2).ok_or("not enough pixel data")?),
			2 => Frame::from_bgra4444(width as u32, height as u32, buf.get(16..16 + width * height * 2).ok_or("not enough pixel data")?),
			5 => Frame::from_rgba_clut4(
				width as u32, height as u32,
				buf.get(16..16 + 1024).ok_or("not enough palette data")?,
				buf.get(16 + 1024..16 + 1024 + width * height / 2).ok_or("not enough pixel data")?
			),
			6 => Frame::from_rgba_clut8(
				width as u32, height as u32,
				buf.get(16..16 + 1024).ok_or("not enough palette data")?,
				buf.get(16 + 1024..16 + 1024 + width * height).ok_or("not enough pixel data")?
			),
			_ => return Err(format!("unhandled PVR pixel format {pixel_fmt}"))
		};
		Ok(Image {frames: Box::new([frame])})
	}
};