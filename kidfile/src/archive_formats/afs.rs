use crate::{file_data::FileData, Certainty, Decoder};
use super::{Archive, ArchiveEntry};

pub const ENTRY_AFS: Decoder<Archive> = Decoder {
	id: "afs",
	desc: "CRI AFS archive used in most KID games",
	detect: |file| Certainty::certain_if(file.starts_with(b"AFS\0")),
	decode: |file| {
		let count = file.get_u32_at(4).map_err(|_| "could not read entry count")? as usize;
		if count >= 0xFFFF {
			return Err("impossibly large entry count".into());
		}
		let mut entries = Vec::with_capacity(count);
		let mut end = 0;
		for i in 0..count {
			let offset = file.get_u32_at(8 + i * 8).map_err(|_| "could not read entry data")?;
			let len = file.get_u32_at(12 + i * 8).map_err(|_| "could not read entry data")?;
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
			let year = file.get_u16_at(pos + 32).map_err(|_| "could not read entry timestamp")?;
			let month = file.get_u16_at(pos + 34).map_err(|_| "could not read entry timestamp")?;
			let day = file.get_u16_at(pos + 36).map_err(|_| "could not read entry timestamp")?;
			let hour = file.get_u16_at(pos + 38).map_err(|_| "could not read entry timestamp")?;
			let minute = file.get_u16_at(pos + 40).map_err(|_| "could not read entry timestamp")?;
			let second = file.get_u16_at(pos + 42).map_err(|_| "could not read entry timestamp")?;
			entries[i].timestamp = Some((year, month, day, hour, minute, second));
		}
		Ok(Archive {format: "afs", entries: entries.into()})
	}
};