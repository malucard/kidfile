use std::borrow::Cow;

use crate::{byte_slice::ByteSlice, image::{Frame, Image, Pixel, PixelFormat}, Certainty, Decoder};

// based on Never7 PS2 decompilation

pub const ENTRY_OGDT: Decoder<Image> = Decoder {
	id: "ogdt",
	desc: "KID PS2 image format",
	detect: |file| Certainty::certain_if(file.starts_with(b"ogdt")),
	decode: |file| {
		let buf = file.read();
		let tile_width = buf.get_u16_at(8).ok_or("could not read width")? as usize;
		let tile_height = buf.get_u16_at(10).ok_or("could not read height")? as usize;
		let column_count = buf.get_u8_at(12).ok_or("could not read column count")? as usize;
		let row_count = buf.get_u8_at(14).ok_or("could not read row count")? as usize;
		let frame_count = column_count * row_count;
		let (fmt, tile_size, clut) = match buf.get_u32_at(4) {
			Some(0) => (
				PixelFormat::Rgba,
				tile_width * tile_height * 4,
				Cow::Borrowed(&[] as &[u8])
			),
			Some(1) => if buf.len() < 32 + frame_count * tile_width * tile_height * 3 {
				(
					PixelFormat::Rgba5551,
					tile_width * tile_height * 2,
					Cow::Borrowed(&[] as &[u8])
				)
			} else {
				(
					PixelFormat::Rgb,
					tile_width * tile_height * 3,
					Cow::Borrowed(&[] as &[u8])
				)
			},
			Some(0x13) => (
				PixelFormat::RgbaClut8,
				tile_width * tile_height,
				Cow::Owned(fix_clut(buf.get(48 + frame_count * tile_width * tile_height..).ok_or("could not read clut8")?))
			),
			Some(0x14) => (
				PixelFormat::RgbaClut4,
				tile_width * tile_height / 2,
				Cow::Borrowed(buf.get(48 + frame_count * tile_width * tile_height / 2..).ok_or("could not read clut4")?)
			),
			Some(x) => return Err(format!("unknown format id 0x{x:X}")),
			None => return Err("could not read format".into())
		};
		let mut final_image = Frame::empty((tile_width * column_count) as u32, (tile_height * row_count) as u32, fmt);
		let mut tile_x = 0;
		let mut tile_y = 0;
		for index in 0..frame_count {
			let tile_start = 32 + index * tile_size;
			let frame_bytes = buf.get(tile_start..tile_start + tile_size).ok_or("could not read pixels")?;
			let tile = match fmt {
				PixelFormat::Rgba => Frame::from_rgba(tile_width as u32, tile_height as u32, frame_bytes).with_double_alpha(),
				PixelFormat::Rgb => Frame::from_rgb(tile_width as u32, tile_height as u32, frame_bytes),
				PixelFormat::Rgba5551 => Frame::from_rgba5551(tile_width as u32, tile_height as u32, frame_bytes),
				PixelFormat::RgbaClut8 => Frame::from_rgba_clut8(tile_width as u32, tile_height as u32, &clut, frame_bytes).with_double_alpha(),
				PixelFormat::RgbaClut4 => Frame::from_rgba_clut4(tile_width as u32, tile_height as u32, &clut, frame_bytes).with_double_alpha(),
				_ => unreachable!()
			};
			final_image.paste(tile_x as u32, tile_y as u32, &tile);
			tile_x += tile_width;
			if tile_x >= tile_width * column_count {
				tile_x = 0;
				tile_y += tile_height;
			}
		}
		Ok(Image {frames: Box::new([final_image])})
	}
};

fn fix_clut(clut: &[u8]) -> Vec<u8> {
	let clut = bytemuck::cast_slice::<u8, Pixel>(clut);
	let mut out = Vec::with_capacity(clut.len());

	const BLOCK_WIDTH: usize = 8;
	const BLOCK_HEIGHT: usize = 2;

	for block_idx in 0..clut.len() / 32 {
		for y in 0..BLOCK_HEIGHT {
			for y2 in 0..BLOCK_HEIGHT {
				for x in 0..BLOCK_WIDTH {
					out.push(clut[block_idx * 32 + y * BLOCK_WIDTH + y2 * BLOCK_WIDTH * BLOCK_HEIGHT + x]);
				}
			}
		}
	}

	bytemuck::cast_vec(out)
}
