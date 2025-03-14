use crate::{byte_slice::ByteSlice, image::Image, Certainty, Decoder};

// https://www.psdevwiki.com/ps3/Graphic_Image_Map_(GIM)

const GIM_BLOCK_ROOT: u16 = 2;
const GIM_BLOCK_PICTURE: u16 = 3;
const GIM_BLOCK_IMAGE: u16 = 4;
const GIM_BLOCK_PALETTE: u16 = 5;
const GIM_BLOCK_FILE_INFO: u16 = 0xFF;

pub const ENTRY_GIM: Decoder<Image> = Decoder {
	id: "gim",
	desc: "PlayStation Portable official image format",
	detect: |file| Certainty::certain_if(file.starts_with(b"MIG\x2E00.1PSP\0")),
	decode: |file| {
		let buf = file.read();
		//let mut frames = Vec::new();
		let mut offset = 16;
		let mut cur_palette = [(0, 0, 0, 0); 256];
		while offset < buf.len() {
			let block_id = buf.read_u16_field(offset, "block id")?;
			//let block_size = buf.read_u32_field(offset + 4, "block size")?;
			let next_block_offset = buf.read_u32_field(offset + 8, "next block offset")? as usize;
			let block_data_offset = buf.read_u32_field(offset + 12, "block data offset")? as usize;
			match block_id {
				GIM_BLOCK_IMAGE => {
					let suboff = offset + block_data_offset;
					let block_data_size = buf.read_u32_field(offset, "block data size")? as usize;

				}
				_ => {}
			}
			offset += next_block_offset;
		}
		todo!()
		// if frames.is_empty() {
		// 	Err("no frames were decoded successfully".into())
		// } else {
		// 	Ok(Image {frames})
		// }
	}
};