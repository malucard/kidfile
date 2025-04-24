use std::mem::MaybeUninit;

use crate::{byte_slice::ByteSlice, file_data::FileData, image::{Frame, Image, Pixel, PixelFormat}, Certainty, Decoder};

// https://www.psxdev.net/forum/viewtopic.php?t=109

#[derive(Debug)]
enum TimFormat {
	Clut4,
	Clut8,
	Psx16,
	Rgb24
}

fn decode_header(data: &mut FileData) -> Option<TimFormat> {
	if data.starts_with(&[16, 0, 0, 0]) {
		let tag = data.get_u32_at(4)?;
		if tag & 0b11 > 1 {
			if tag | 0b11 != 0b11 {
				return None;
			}
		} else {
			if tag | 0b11 != 0b1011 {
				return None;
			}
		}
		match tag & 3 {
			0 => Some(TimFormat::Clut4),
			1 => Some(TimFormat::Clut8),
			2 => Some(TimFormat::Psx16),
			3 => Some(TimFormat::Rgb24),
			_ => None
		}
	} else {
		None
	}
}

fn psx_to_rgba(color: u16) -> Pixel {
	Pixel {
		r: ((color << 3) & 0b11111000 | (color & 0b111)) as u8,
		g: ((color >> 2) & 0b11111000 | (color >> 5 & 0b111)) as u8,
		b: ((color >> 7) & 0b11111000 | (color >> 10 & 0b111)) as u8,
		a: if color == 0 {0} else {255}
	}
}

pub const ENTRY_TIM: Decoder<Image> = Decoder {
	id: "tim",
	desc: "PlayStation 1 official image format",
	detect: |file| Certainty::certain_if(decode_header(file).is_some()),
	decode: |file| {
		let header = decode_header(file).ok_or("could not decode header")?;
		let buf = file.read();
		match header {
			TimFormat::Clut4 => {
				let clut = bytemuck::cast_slice::<u8, u16>(&buf[20..20 + 16 * 2]);
				let pixel_start = 20 + 16 * 2 + 12;
				let vram_width = buf.read_u16(pixel_start - 4)? as usize;
				let height = buf.read_u16(pixel_start - 2)? as usize;
				if buf.len() < pixel_start + vram_width * height {
					return Err("not enough pixels".into());
				}
				let pixel_width = vram_width * 4;
				let pixel_count = pixel_width * height;
				let mut pixels = unsafe {Box::new_uninit_slice(pixel_count).assume_init()};
				for i in 0..pixel_count / 2 {
					let two_pixels = buf[pixel_start + i] as usize;
					pixels[i * 2] = psx_to_rgba(clut[(two_pixels & 0xF) as usize]);
					pixels[i * 2 + 1] = psx_to_rgba(clut[(two_pixels >> 4) as usize]);
				}
				Ok(Image {frames: Box::new([Frame {
					width: pixel_width as u32,
					height: height as u32,
					og_fmt: PixelFormat::PsxClut4,
					pixels
				}])})
			}
			TimFormat::Clut8 => {
				let clut = bytemuck::cast_slice::<u8, u16>(&buf[20..20 + 256 * 2]);
				let pixel_start = 20 + 256 * 2 + 12;
				let vram_width = buf.read_u16(pixel_start - 4)? as usize;
				let height = buf.read_u16(pixel_start - 2)? as usize;
				if buf.len() < pixel_start + vram_width * height {
					return Err("not enough pixels".into());
				}
				let pixel_width = vram_width * 2;
				let pixel_count = pixel_width * height;
				let mut pixels = unsafe {Box::new_uninit_slice(pixel_count).assume_init()};
				for i in 0..pixel_count {
					pixels[i] = psx_to_rgba(clut[buf[pixel_start + i] as usize]);
				}
				Ok(Image {frames: Box::new([Frame {
					width: pixel_width as u32,
					height: height as u32,
					og_fmt: PixelFormat::PsxClut8,
					pixels
				}])})
			}
			TimFormat::Rgb24 => {
				let pixel_start = 20;
				let vram_width = buf.read_u16(pixel_start - 4)? as usize;
				let height = buf.read_u16(pixel_start - 2)? as usize;
				if buf.len() < pixel_start + vram_width * height {
					return Err("not enough pixels".into());
				}
				let pixel_width = vram_width / 3;
				Ok(Image {frames: Box::new([Frame::from_rgb(
					pixel_width as u32, height as u32,
					&buf[pixel_start..pixel_start + vram_width * height]
				)])})
			}
			_ => Err(format!("todo {:?}", header))
		}
	}
};