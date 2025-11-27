use std::borrow::Cow;

use crate::{byte_slice::ByteSlice, image::{Frame, Image, Pixel, PixelFormat}, Certainty, Decoder};

pub const ENTRY_PRT: Decoder<Image> = Decoder {
	id: "prt",
	desc: "Old KID PC image format",
	detect: |file| Certainty::certain_if(file.starts_with(b"PRT\0") || file.starts_with_at(b"PRT\0", 16)),
	decode: |file| {
		let buf = file.read();
		let buf = if buf.starts_with(b"PRT\0") {
			&buf
		} else {
			&buf[16..]
		};
		let version = buf.read_u16(4)?;
		if version != 101 && version != 102 {
			return Err(format!("unsupported prt version {version}"));
		}
		let bpp = buf.read_u16(6)? as u32;
		let palette_pos = buf.read_u16(8)? as usize;
		let pixel_pos = buf.read_u16(10)? as usize;
		let has_alpha = buf.read_u32(16)? != 0;
		let img_w;
		let img_h;
		if version == 102 {
			//offx = buf.read_u32(20)?; // what is this?
			//offy = buf.read_u32(24)?;
			img_w = buf.read_u32(28)?;
			img_h = buf.read_u32(32)?;
		} else {
			img_w = buf.read_u16(12)? as u32;
			img_h = buf.read_u16(14)? as u32;
		}
		let stride = (img_w * (bpp / 8) + 3) / 4 * 4;
		if bpp == 8 {
			let mut frame = Frame::empty(img_w, img_h, PixelFormat::BgrxClut8);
			for y in (0..img_h).rev() {
				let row_pos = pixel_pos + (stride * y) as usize;
				let row = Frame::from_bgrx_clut8(img_w, 1, &buf[palette_pos..], &buf[row_pos..]);
				frame.paste(0, img_h - y - 1, &row);
			}
			Ok(Image {frames: Box::new([frame])})
		} else if bpp == 24 {
			let mut frame = Frame::empty(img_w, img_h, PixelFormat::Bgra);
			let mut alpha_pos = pixel_pos + (stride * img_h) as usize;
			let mut i = 0;
			for y in (0..img_h).rev() {
				let mut row_pos = pixel_pos + (stride * y) as usize;
				for _ in 0..img_w {
					frame.pixels[i] = Pixel {
						r: buf[row_pos + 2],
						g: buf[row_pos + 1],
						b: buf[row_pos],
						a: if has_alpha {buf[alpha_pos]} else {255}
					};
					row_pos += 3;
					alpha_pos += 1;
					i += 1;
				}
			}
			Ok(Image {frames: Box::new([frame])})
		} else {
			Err(format!("unexpected bpp of {bpp}"))
		}
	}
};
