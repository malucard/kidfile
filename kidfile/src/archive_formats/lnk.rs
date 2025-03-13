use crate::{file_data::FileData, Certainty, Decoder};
use super::{Archive, ArchiveEntry};

pub const ENTRY_LNK: Decoder<Archive> = Decoder {
	id: "lnk",
	desc: "KID PC archive",
	detect: |file| Certainty::certain_if(file.starts_with(b"LNK\0")),
	decode: |file| {
		let count = file.get_u32_at(4).map_err(|_| "could not read signature")? as usize;
		if count >= 0xFFFF {
			return Err("impossibly large entry count".into());
		}
		let mut entries = Vec::with_capacity(count);
		let mut index_ptr = 16;
		let mut data_section_start = 16 + count * 32;
		for _ in 0..count {
			let offset = file.get_u32_at(index_ptr).map_err(|_| "could not read entry data")?;
			let mut len = file.get_u32_at(index_ptr + 4).map_err(|_| "could not read entry data")?;
			let is_compressed = len & 1 != 0;
			len >>= 1;
			let mut name_buf = [0u8; 24];
			file.read_chunk_exact(&mut name_buf, index_ptr + 8).map_err(|_| "could not read entry name")?;
			let name_len = name_buf.iter().position(|x| *x == 0).unwrap_or(32);
			entries.push(ArchiveEntry {
				name: std::str::from_utf8(&name_buf[0..name_len]).map_err(|_| "entry name is not valid UTF-8")?.into(),
				data: file.subfile(data_section_start + offset as usize, len as usize).unwrap(),
				timestamp: None
			});
			index_ptr += 32;
		}
		Ok(Archive {format: "lnk", entries: entries.into()})
	}
};