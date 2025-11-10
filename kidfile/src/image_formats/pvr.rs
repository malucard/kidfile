use std::{fs::File, io::{Read, Seek, SeekFrom}};

use crate::{Certainty, Decoder, byte_slice::ByteSlice, image::{Frame, Image, PixelFormat, bit_twiddle}};

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
		let mut file_start = 0;
		let mut tex_start = 0;
		let mut tex_len = 0;
		let mut palettes = Vec::new();
		while file_start < file.len() {
			let file_len = file.read_u32(file_start + 4)? as usize;
			if file.starts_with_at(b"PVPL", file_start) {
				let mut palette_bytes = Default::default();
				palette_bytes = unsafe {Box::new_uninit_slice(file_len - 8).assume_init()};
				file.read_chunk_exact(&mut palette_bytes, file_start + 16).map_err(|_| "PVPL length field is incorrect")?;
				palettes.push(palette_bytes);
			} else if file.starts_with_at(b"PVRT", file_start) {
				tex_start = file_start;
				tex_len = file_len;
			}
			file_start += file_len + 8;
		}
		if tex_len <= 0 {
			return Err("PVRT header not found in file".to_string());
		}
		let mut buf = unsafe {Box::new_uninit_slice(tex_len + 8).assume_init()};
		file.read_chunk_exact(&mut buf, tex_start).map_err(|_| "PVRT length field is incorrect")?;
		let pixel_fmt = buf.read_u8(8)?;
		let twiddle_type = buf.read_u8(9)?;
		println!("twiddle type {twiddle_type}");
		let width = buf.read_u16(12)? as usize;
		let height = buf.read_u16(14)? as usize;
		let mut frames = Vec::new();
		if palettes.len() == 0 {
			let palette_bytes = unsafe {Box::new_uninit_slice(0).assume_init()};
			palettes.push(palette_bytes);
		}
		for palette_bytes in palettes.iter() {
			let mut frame = match pixel_fmt {
				0 | 1 | 2 => {
					if twiddle_type == 3 { // vq compression
						let codebook = buf.get(16..16 + 2048).ok_or("not enough 16-bit VQ codebook data")?;
						let indices = buf.get(16 + 2048..16 + 2048 + width * height / 4).ok_or("not enough VQ index data")?;
						let mut pixels = vec![0u8; width * height * 2];
						for block_y in 0..height / 2 {
							let twiddled_block_y = bit_twiddle(block_y);
							for block_x in 0..width / 2 {
								let twiddled_block_idx = twiddled_block_y | bit_twiddle(block_x) << 1;
								let codebook_pos = indices[twiddled_block_idx] as usize * 8;
								let x = block_x * 2;
								let y = block_y * 2;
								pixels[(y * width + x) * 2] = codebook[codebook_pos];
								pixels[(y * width + x) * 2 + 1] = codebook[codebook_pos + 1];
								pixels[(y * width + x + 1) * 2] = codebook[codebook_pos + 4];
								pixels[(y * width + x + 1) * 2 + 1] = codebook[codebook_pos + 5];
								pixels[((y + 1) * width + x) * 2] = codebook[codebook_pos + 2];
								pixels[((y + 1) * width + x) * 2 + 1] = codebook[codebook_pos + 3];
								pixels[((y + 1) * width + x + 1) * 2] = codebook[codebook_pos + 6];
								pixels[((y + 1) * width + x + 1) * 2 + 1] = codebook[codebook_pos + 7];
							}
						}
						if pixel_fmt == 0 {
							Frame::from_bgra5551(width as u32, height as u32, &pixels).with_og_fmt(PixelFormat::Bgra5551Vq8)
						} else if pixel_fmt == 1 {
							Frame::from_bgr565(width as u32, height as u32, &pixels).with_og_fmt(PixelFormat::Bgr565Vq8)
						} else {
							Frame::from_bgra4444(width as u32, height as u32, &pixels).with_og_fmt(PixelFormat::Bgra4444Vq8)
						}
					} else {
						if pixel_fmt == 0 {
							Frame::from_bgra5551(width as u32, height as u32, buf.get(16..16 + width * height * 2).ok_or("not enough pixel data for BGRA5551")?)
						} else if pixel_fmt == 1 {
							Frame::from_bgr565(width as u32, height as u32, buf.get(16..16 + width * height * 2).ok_or("not enough pixel data for BGR565")?)
						} else {
							Frame::from_bgra4444(width as u32, height as u32, buf.get(16..16 + width * height * 2).ok_or("not enough pixel data for BGRA4444")?)
						}
					}
				}
				5 => {
					if twiddle_type == 7 && palette_bytes.is_empty(){
						return Err("file needs external palette, unimplemented".into());
					} else if palette_bytes.is_empty() {
						Frame::from_bgra_clut4(
							width as u32, height as u32,
							buf.get(16..16 + 1024).ok_or("not enough palette data for BGRA clut4")?,
							buf.get(16 + 1024..16 + 1024 + width * height / 2).ok_or("not enough index data for BGRA clut4")?
						)
					} else {
						Frame::from_bgra_clut4(
							width as u32, height as u32,
							&palette_bytes,
							buf.get(16..16 + width * height / 2).ok_or("not enough data for BGRA clut4")?
						)
					}
				}
				6 => {
					if twiddle_type == 7 && palette_bytes.is_empty(){
						return Err("file needs external palette, unimplemented".into());
					} else if palette_bytes.is_empty() {
						Frame::from_bgra_clut8(
							width as u32, height as u32,
							buf.get(16..16 + 1024).ok_or("not enough palette data for BGRA clut8")?,
							buf.get(16 + 1024..16 + 1024 + width * height).ok_or("not enough index data for BGRA clut8")?
						)
					} else {
						Frame::from_bgra_clut8(
							width as u32, height as u32,
							&palette_bytes,
							buf.get(16..16 + width * height).ok_or("not enough data for BGRA clut8")?
						)
					}
				}
				_ => return Err(format!("unhandled PVR pixel format {pixel_fmt}"))
			};
			if [1, 2, 5, 6, 7, 8, 13].contains(&twiddle_type) {
				frame = frame.twiddled_dc();
			}
			frames.push(frame);
		}
		Ok(Image {frames: frames.into_boxed_slice()})
	}
};
