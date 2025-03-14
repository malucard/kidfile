use crate::{file_data::FileData, Certainty, Decoder};
use super::{Archive, ArchiveEntry};

pub const ENTRY_AFS: Decoder<Archive> = Decoder {
	id: "afs",
	desc: "CRI AFS archive used in most KID games",
	detect: |file| Certainty::certain_if(file.starts_with(b"AFS\0")),
	decode: |file| {
		let count = file.read_u32_field(4, "entry count")? as usize;
		if count >= 0xFFFF {
			return Err("impossibly large entry count".into());
		}
		let mut entries = Vec::with_capacity(count);
		let mut end = 0;
		for i in 0..count {
			let offset = file.read_u32_field(8 + i * 8, "entry offset")?;
			let len = file.read_u32_field(12 + i * 8, "entry length")?;
			end = end.max(offset + len);
			entries.push(ArchiveEntry {
				name: String::new(),
				data: file.subfile(offset as usize, len as usize).unwrap(),
				timestamp: None
			});
		}
		end = end.next_multiple_of(0x800);
		for i in 0..count {
			let pos = end as usize + i * 48;
			let mut name_buf = [0u8; 32];
			file.read_chunk_exact(&mut name_buf, pos).map_err(|_| "could not read entry name")?;
			let len = name_buf.iter().position(|x| *x == 0).unwrap_or(32);
			entries[i].name = std::str::from_utf8(&name_buf[0..len]).map_err(|_| "entry name is not valid UTF-8")?.into();
			let year = file.read_u16_field(pos + 32, "entry timestamp")?;
			let month = file.read_u16_field(pos + 34, "entry timestamp")?;
			let day = file.read_u16_field(pos + 36, "entry timestamp")?;
			let hour = file.read_u16_field(pos + 38, "entry timestamp")?;
			let minute = file.read_u16_field(pos + 40, "entry timestamp")?;
			let second = file.read_u16_field(pos + 42, "entry timestamp")?;
			entries[i].timestamp = Some((year, month, day, hour, minute, second));
		}
		Ok(Archive {format: "afs", entries: entries.into()})
	}
};