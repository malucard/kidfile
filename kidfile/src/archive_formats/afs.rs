use crate::{Certainty, Decoder};
use super::{Archive, ArchiveEntry};

pub const ENTRY_AFS: Decoder<Archive> = Decoder {
	id: "afs",
	desc: "CRI AFS archive used in most KID games",
	detect: |file| Certainty::certain_if(file.starts_with(b"AFS\0")),
	decode: |file| {
		let count = file.read_u32(4)? as usize;
		if count >= 0xFFFF {
			return Err("impossibly large entry count".into());
		}
		let mut entry_ranges = Vec::with_capacity(count);
		let mut entries = Vec::with_capacity(count);
		let mut end = 0;
		for i in 0..count {
			let offset = file.read_u32(8 + i * 8)? as usize;
			let len = file.read_u32(12 + i * 8)? as usize;
			end = end.max(offset + len);
			entry_ranges.push((offset, len));
		}
		end = end.next_multiple_of(0x800);
		for i in 0..count {
			let pos = end as usize + i * 48;
			let mut name_buf = [0u8; 32];
			file.read_chunk_exact(&mut name_buf, pos).map_err(|_| "could not read entry name")?;
			let len = name_buf.iter().position(|x| *x == 0).unwrap_or(32);
			let year = file.read_u16(pos + 32)?;
			let month = file.read_u16(pos + 34)?;
			let day = file.read_u16(pos + 36)?;
			let hour = file.read_u16(pos + 38)?;
			let minute = file.read_u16(pos + 40)?;
			let second = file.read_u16(pos + 42)?;
			let name = String::from_utf8(name_buf[0..len].to_vec()).map_err(|_| "entry name is not valid UTF-8")?;
			entries.push(ArchiveEntry {
				name: name.clone(),
				data: file.subfile(entry_ranges[i].0, entry_ranges[i].1).unwrap(),
				timestamp: Some((year, month, day, hour, minute, second))
			});
		}
		Ok(Archive {format: "afs", entries: entries.into()})
	}
};