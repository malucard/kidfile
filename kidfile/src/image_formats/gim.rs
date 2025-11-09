use std::borrow::Cow;

use bytemuck::Zeroable;
use crate::{byte_slice::ByteSlice, image::{Frame, Image, Pixel}, Certainty, Decoder};

// https://www.psdevwiki.com/ps3/Graphic_Image_Map_(GIM)

struct GimBlock {
	next: usize,
	next_skipping_children: usize,
	id: u16,
	data_start: usize
}

impl GimBlock {
	pub fn parse(buf: &[u8], pos: usize) -> Result<Self, String> {
		Ok(Self {
			next: pos + buf.read_u32(pos + 8)? as usize,
			next_skipping_children: pos + buf.read_u32(pos + 4)? as usize,
			id: buf.read_u16(pos)?,
			data_start: pos + buf.read_u32(pos + 12)? as usize
		})
	}
}

pub const ENTRY_GIM: Decoder<Image> = Decoder {
	id: "gim",
	desc: "PlayStation Portable official image format",
	detect: |file| Certainty::certain_if(file.starts_with(b"MIG\x2E00.1PSP\0")),
	decode: |file| {
		let buf = file.read();
		let mut frames = Vec::new();
		let mut pos = 16;
		let mut cur_palette: &[u8] = &[];
		while pos < buf.len() {
			let block = GimBlock::parse(buf, pos)?;
			if block.id == 3 { // picture block (children: image, palette)
				// search for a palette, so that we already have it set when getting to the image block
				let mut child_pos = block.data_start;
				while child_pos < block.next_skipping_children {
					let child_block = GimBlock::parse(buf, child_pos)?;
					if child_block.id == 5 { // palette block
						let palette_start = child_block.data_start + buf.read_u32(child_block.data_start + 28)? as usize;
						cur_palette = &buf[palette_start..palette_start + 1024];
						break;
					}
					child_pos = child_block.next;
				}
			} else if block.id == 4 { // image block
				let format = buf.read_u16(block.data_start + 4)? as usize;
				let swizzled = buf.read_u16(block.data_start + 6)? != 0;
				let width = buf.read_u16(block.data_start + 8)? as u32;
				let height = buf.read_u16(block.data_start + 10)? as u32;
				let width_alignment = buf.read_u16(block.data_start + 14)? as u32;
				let format_bpp = match format {
					0 => 16,
					1 => 16,
					3 => 32,
					4 => 4,
					5 => 8,
					x => return Err(format!("unhandled pixel format {x:#X}"))
				};
				let aligned_width = width.next_multiple_of(width_alignment * 8 / format_bpp);
				let pixel_start = block.data_start + buf.read_u32(block.data_start + 28)? as usize;
				let pixel_data = if swizzled {
					Cow::Owned(buf[pixel_start..].unswizzled_psp(aligned_width * format_bpp / 8, height))
				} else {
					Cow::Borrowed(&buf[pixel_start..])
				};
				let frame = match format {
					0 => Frame::from_rgb16(aligned_width, height, &pixel_data),
					1 => Frame::from_rgba5551(aligned_width, height, &pixel_data),
					3 => Frame::from_rgba(aligned_width, height, &pixel_data),
					4 => Frame::from_rgba_clut4(aligned_width, height, cur_palette, &pixel_data),
					5 => Frame::from_rgba_clut8(aligned_width, height, cur_palette, &pixel_data),
					x => return Err(format!("unhandled pixel format {x:#X}"))
				};
				frames.push(frame.resized(width, height));
			}
			pos = block.next;
		}
		if frames.is_empty() {
			Err("no frames were decoded successfully".into())
		} else {
			Ok(Image {frames: frames.into_boxed_slice()})
		}
	}
};
