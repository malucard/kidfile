use zune_inflate::{DeflateDecoder, DeflateOptions};
use crate::{byte_slice::ByteSlice, image::{Frame, Image}, Certainty, Decoder};

pub const ENTRY_KLZ: Decoder<Image> = Decoder {
	id: "klz",
	desc: "12Riven bizarre image format",
	detect: |file| Certainty::certain_if(file.starts_with(b"TIM2") && file.starts_with_at(b"PNGFILE3", 0x40)),
	decode: |file| {
		let mut frames = Vec::new();
		let bytes = file.read();
		let mut entry_start = 0; // KLZ files often have multiple entries, they seem to just be concatenated
		while bytes.len() > entry_start {
			// at address 16 in the TIM2 header, there is a 32-bit number equal to (file size - 16), ie, the size starting from this number itself
			// at address 24 in the GXT5 TIM2 header, there is a 32-bit number equal to (file size - 64), ie, the size starting from the PNGFILE3 header
			// in FXT5 that seems to not include the palette, so we could use that to find the compressed pixel data and the palette separately
			// since the palette size is always 1024 bytes though, i just do a little math instead
			// the PNGFILE3 header is at 64 and is 124 bytes long
			// the inner file contents start at 188
			let entry_size = bytes.get_u32_at(entry_start + 16).ok_or("could not read entry size")? as usize + 16;
			if bytes.len() < entry_start + entry_size {
				return Err(format!("expected {} bytes, had only {}", entry_size, bytes.len() - entry_start));
			}
			let subformat = bytes.get(entry_start + 164..entry_start + 168).ok_or("could not read subformat")?;
			if &subformat == b"GXT5" {
				// this format is just a PNG with nothing special about it
				let mut decoder = png::Decoder::new(&bytes[entry_start + 188..entry_start + entry_size]);
				decoder.set_transformations(png::Transformations::normalize_to_color8().union(png::Transformations::ALPHA));
				let mut reader = decoder.read_info().map_err(|e| format!("in GXT5 PNG info: {}", e))?;
				let mut buf = vec![0u8; reader.output_buffer_size()];
				let info = reader.next_frame(buf.as_mut()).map_err(|e| format!("in GXT5 PNG frame: {}", e))?;
				assert_eq!(info.buffer_size(), (info.width * info.height * 4) as usize);
				buf.truncate(info.buffer_size());
				frames.push(Frame::from_rgba(info.width, info.height, &buf));
			} else if &subformat == b"FXT5" {
				// this is an 8-bit palette format, with 256x R8G8B8A8 palette entries, where the palette needs to be shuffled in a certain way because of PS2 hardware
				// pixels: [u8; width * height]
				// palette: [[u8; 4]; 256]
				let compressed_size = entry_size - 188 - 256 * 4;
				let expected_size = bytes.get_u32_at(entry_start + 156).ok_or("could not read FXT expected uncompressed size")? as usize;
				let width = bytes.get_u32_at(entry_start + 180).ok_or("could not read FXT5 width")?;
				let height = bytes.get_u32_at(entry_start + 184).ok_or("could not read FXT5 height")?;
				let palette_start = entry_start + 188 + compressed_size;
				match DeflateDecoder::new_with_options(
					&bytes.get(entry_start + 188..palette_start).ok_or("could not read FXT5 compressed pixel section")?,
					DeflateOptions::default().set_limit(expected_size).set_size_hint(expected_size)
				).decode_zlib() {
					Ok(pixel_bytes) => {
						let mut palette = [[0u8; 4]; 256];
						bytemuck::cast_slice_mut::<[u8; 4], u8>(&mut palette).copy_from_slice(bytes.get(palette_start..palette_start + 256 * 4).ok_or("could not read FXT5 palette")?);
						// PS2 palette shift
						for i in 0..8 {
							for j in 0..8 {
								let tmp = palette[8 + i * 32 + j];
								palette[8 + i * 32 + j] = palette[16 + i * 32 + j];
								palette[16 + i * 32 + j] = tmp;
							}
						}
						// for some reason the alpha is 7-bit, so this fixes it
						for color in palette.iter_mut() {
							color[3] = color[3] << 1 | (color[3] & 1);
						}
						let mut pixels = Vec::new();
						pixels.reserve_exact((width * height) as usize);
						for i in 0..(width * height) as usize {
							pixels.push(palette[pixel_bytes[i] as usize]);
						}
						frames.push(Frame::from_rgba(width, height, bytemuck::cast_slice(&pixels)));
					}
					Err(e) => return Err(format!("error decompressing FXT5 pixel section: {}", e))
				}
			} else {
				// apparently the raw PNGs don't always have GXT5, and instead just some garbage, but those seem to be specifically the BGRA images
				let mut decoder = png::Decoder::new(&bytes[entry_start + 188..entry_start + entry_size]);
				decoder.set_transformations(png::Transformations::normalize_to_color8().union(png::Transformations::ALPHA));
				let mut reader = decoder.read_info().map_err(|e| format!("error reading BGRA PNG info: {e}"))?;
				let mut buf = vec![0u8; reader.output_buffer_size()];
				let info = reader.next_frame(&mut buf).map_err(|e| format!("error reading BGRA PNG frame: {e}"))?;
				assert_eq!(info.buffer_size(), (info.width * info.height * 4) as usize);
				for pixel in bytemuck::cast_slice_mut::<u8, [u8; 4]>(&mut buf) {
					let tmp = pixel[0];
					pixel[0] = pixel[2];
					pixel[2] = tmp;
					pixel[3] = pixel[3] << 1 | (pixel[3] & 1);
				}
				buf.truncate(info.buffer_size());
				frames.push(Frame::from_rgba(info.width, info.height, &buf));
				//let nul_pos = subformat.iter().position(|x| *x == 0).unwrap_or(4);
				//return Err(format!("Unknown subformat \"{}\"", std::str::from_utf8(&subformat[..nul_pos]).unwrap_or_default()));
			}
			entry_start += entry_size;
		}
		if frames.is_empty() {
			Err("no frames in image".into())
		} else {
			Ok(Image {frames: frames.into_boxed_slice()})
		}
	}
};