use crate::{byte_slice::ByteSlice, image::{Frame, Image, PixelFormat}, Certainty, Decoder};

pub const ENTRY_BIP: Decoder<Image> = Decoder {
	id: "bip",
	desc: "Remember11 image format",
	detect: |file| Certainty::certain_if(file.starts_with(&[5, 0, 0, 0]) || file.starts_with(&[10, 0, 0, 0])),
	decode: |file| {
		let mut frames = Vec::new();
		let bytes = file.read();

		let header_end = bytes.read_u32(0)? as usize * 4;
		let mut palette_section = bytes.read_u32(header_end - 8)? as usize;
		let pixel_section = bytes.read_u32(header_end - 4)? as usize;
		let mut index_section = bytes.read_u32(4)? as usize;
		let mut src_block_x_idx = 0;
		let mut src_block_y_idx = 0;
		loop {
			let tile_count = bytes.read_u16(index_section)? as usize;
			if tile_count == 0 {
				break;
			}
			let og_full_width = bytes.read_u16(index_section + 8)? as u32;
			let og_full_height = bytes.read_u16(index_section + 10)? as u32;
			let mut cur_frame: Option<Frame> = None;
			index_section += 12;
			let mut next_palette_section = 0;
			for tile_idx in 0..tile_count {
				let tile_size = bytes.read_u16(index_section)? as usize;
				match tile_size {
					0 => break,
					2 => { // RGBA clut8
						let tile_x = bytes.read_u8(index_section + 4)? as u32 * 30;
						let tile_y = bytes.read_u8(index_section + 5)? as u32 * 30;
						let tile_x_blocks = bytes.read_u8(index_section + 6)? as u32;
						let tile_y_blocks = bytes.read_u8(index_section + 7)? as u32;
						let palette = &bytes[palette_section..];
						//let tile_pixel_data = pixel_section; // + bytes.read_u16(index_section + 2)? as usize * 1024;
						let frame = cur_frame.get_or_insert_with(|| Frame::empty(og_full_width, og_full_height, PixelFormat::RgbaClut8));
						// this part is SUPER quirky. the source image is split into 30x30 blocks, but the first and last rows and columns are
						// repeated so the blocks grow to 32x32. however, the blocks aren't stored sequentially, but rather as an image
						// with its width forced to 512. if the real width is 512, it can be directly read as an image but the
						// repeated pixels in the blocks will be visible. if it's not 512 the rows of blocks wrap and that breaks.
						// the simplest way to work with that mess is to iterate through the blocks and calculate the indices separately
						let mut dst_block_x_idx = 0;
						let mut dst_block_y_idx = 0;
						for _ in 0..tile_x_blocks * tile_y_blocks {
							let mut src_block_start = pixel_section + (src_block_y_idx * (512 * 32) + 512 + src_block_x_idx * 32) as usize;
							let dst_y_start = tile_y + dst_block_y_idx * 30;
							for dst_y in dst_y_start..dst_y_start + 30 {
								if src_block_start + 30 >= bytes.len() {
									break;
								}
								let row = Frame::from_rgba_clut8(30, 1, palette, &bytes[src_block_start + 1..]);
								frame.paste(tile_x + dst_block_x_idx * 30, dst_y, &row);
								src_block_start += 512;
							}
							src_block_x_idx += 1;
							if src_block_x_idx >= 16 {
								src_block_x_idx = 0;
								src_block_y_idx += 1;
							}
							dst_block_x_idx += 1;
							if dst_block_x_idx >= tile_x_blocks {
								dst_block_x_idx = 0;
								dst_block_y_idx += 1;
							}
						}
						next_palette_section = palette_section + 1024;
					}
					7 => { // png (?)
						let png_full_width = bytes.read_u16(index_section + 20)? as u32;
						let png_full_height = bytes.read_u16(index_section + 22)? as u32;
						let tile_x = bytes.read_u16(index_section + 8)? as u32;
						let tile_y = bytes.read_u16(index_section + 10)? as u32;
						let tile_pixel_data = pixel_section + bytes.read_u32(index_section + 24)? as usize;

						let tile_data_end = tile_pixel_data + 8 + bytes.read_u32(tile_pixel_data + 32)? as usize;
						let tile_x_off = bytes.read_u32(tile_pixel_data + 116)?;
						let tile_y_off = bytes.read_u32(tile_pixel_data + 120)?;

						let mut decoder = png::Decoder::new(&bytes[tile_pixel_data + 132..tile_data_end]);
						decoder.set_transformations(png::Transformations::normalize_to_color8().union(png::Transformations::ALPHA));
						let mut reader = decoder.read_info().map_err(|e| format!("in PNGFILE2 PNG info: {}", e))?;
						let mut bgra_buf = vec![0u8; reader.output_buffer_size()];
						let info = reader.next_frame(bgra_buf.as_mut()).map_err(|e| format!("in PNGFILE2 PNG frame: {}", e))?;
						assert_eq!(info.buffer_size(), (info.width * info.height * 4) as usize);

						let frame = cur_frame.get_or_insert_with(|| Frame::empty(png_full_width, png_full_height, reader.info().into()));
						let tile = Frame::from_bgra(info.width, info.height, &bgra_buf).with_double_alpha();
						frame.paste(tile_x + tile_x_off, tile_y + tile_y_off, &tile);
					}
					_ => return Err(format!("unhandled bip tile index size {tile_size}"))
				}
				index_section += tile_size * 4;
			}
			palette_section = next_palette_section;
			if let Some(frame) = cur_frame.take() {
				frames.push(frame);
			}
		}
		if frames.is_empty() {
			Err("no frames decoded".into())
		} else {
			Ok(Image {frames: frames.into_boxed_slice()})
		}
	}
};