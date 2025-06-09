use crate::{byte_slice::ByteSlice, image::{Frame, Image, PixelFormat}, Certainty, Decoder};

pub const ENTRY_BIP: Decoder<Image> = Decoder {
	id: "bip",
	desc: "Remember11 image format",
	detect: |file| Certainty::certain_if(matches!(file.get_u32_at(0), Some(5 | 10))),
	decode: |file| {
		let bytes = file.read();
		let mut is_remember11 = false;
		'retry: loop {
			let mut frames = Vec::new();
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
				let is_paletted = bytes.read_u16(index_section + 2)? as u32 != 0;
				let pixel_bytes = if is_paletted {1} else {4};
				let pseudo_block_size = if is_paletted {32} else {16};
				let real_block_size = if is_paletted {30} else {if is_remember11 {14} else {16}};
				let og_full_width = bytes.read_u16(index_section + 8)? as u32;
				let og_full_height = bytes.read_u16(index_section + 10)? as u32;
				let mut cur_frame: Option<Frame> = None;
				index_section += 12;
				let mut next_palette_section = 0;
				for _ in 0..tile_count {
					let tile_size = bytes.read_u16(index_section)? as usize;
					match tile_size {
						0 => break,
						2 => { // RGBA clut8
							let tile_x = bytes.read_u8(index_section + 4)? as u32 * real_block_size;
							let tile_y = bytes.read_u8(index_section + 5)? as u32 * real_block_size;
							let tile_x_blocks = bytes.read_u8(index_section + 6)? as u32;
							let tile_y_blocks = bytes.read_u8(index_section + 7)? as u32;
							// this is a heuristic, but it seems to work well: if the tile is past the borders beyond what's necessary, change the block size and retry
							// this is because R11 non-paletted images have the repeated lines in each block (as described below), but E17 ones don't
							if tile_x + tile_x_blocks * real_block_size >= og_full_width + real_block_size || tile_y + tile_y_blocks * real_block_size >= og_full_height + real_block_size {
								if is_remember11 {
									return Err("tile overflow".into());
								} else {
									is_remember11 = true;
									continue 'retry;
								}
							}
							let palette = &bytes[palette_section..];
							let frame = cur_frame.get_or_insert_with(|| {
								Frame::empty(og_full_width, og_full_height, if is_paletted {PixelFormat::RgbaClut8} else {PixelFormat::Rgba})
							});
							// this part is SUPER quirky. the source image is split into 30x30 blocks, but the first and last rows and columns are
							// repeated so the blocks grow to 32x32. however, the blocks aren't stored sequentially, but rather as an image
							// with its width forced to 512. if the real width is 512, it can be directly read as an image but the
							// repeated pixels in the blocks will be visible. if it's not 512 the rows of blocks wrap and that breaks.
							// the simplest way to work with that mess is to iterate through the blocks and calculate the indices separately
							// edit: when the image isn't paletted, the block size is 16 instead of 32, and E17 PS2 doesn't have those repeated lines, but R11 does
							let mut dst_block_x_idx = 0;
							let mut dst_block_y_idx = 0;
							for _ in 0..tile_x_blocks * tile_y_blocks {
								let mut src_block_start = pixel_section + (src_block_y_idx * (512 * pseudo_block_size) + src_block_x_idx * pseudo_block_size) as usize * pixel_bytes;
								if real_block_size != pseudo_block_size {
									src_block_start += 513 * pixel_bytes;
								}
								let dst_y_start = tile_y + dst_block_y_idx * real_block_size;
								for dst_y in dst_y_start..dst_y_start + real_block_size {
									if src_block_start + real_block_size as usize * pixel_bytes >= bytes.len() {
										break;
									}
									let row = if is_paletted {
										Frame::from_rgba_clut8(real_block_size, 1, palette, &bytes[src_block_start..])
									} else {
										Frame::from_rgba(real_block_size, 1, &bytes[src_block_start..]).with_double_alpha()
									};
									frame.paste(tile_x + dst_block_x_idx * real_block_size, dst_y, &row);
									src_block_start += 512 * pixel_bytes;
								}
								src_block_x_idx += 1;
								if src_block_x_idx >= 512 / pseudo_block_size {
									src_block_x_idx = 0;
									src_block_y_idx += 1;
								}
								dst_block_x_idx += 1;
								if dst_block_x_idx >= tile_x_blocks {
									dst_block_x_idx = 0;
									dst_block_y_idx += 1;
								}
							}
							if is_paletted {
								next_palette_section = palette_section + 1024;
							}
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
				return Err("no frames decoded".into());
			} else {
				return Ok(Image {frames: frames.into_boxed_slice()});
			}
		}
	}
};
