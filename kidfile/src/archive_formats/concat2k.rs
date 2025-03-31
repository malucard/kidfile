use crate::{Certainty, Decoder};
use super::{Archive, ArchiveEntry};

// it's annoying that this has to exist, but here we go
// files on N7 PS2 are literally just aligned and concatenated, without even file name info
// here we simply use the heuristic of checking for TIM2 and OGDT signatures every 2048 bytes

const ALIGNMENT: usize = 2048;

pub const ENTRY_CONCAT2K: Decoder<Archive> = Decoder {
	id: "concat2k",
	desc: "Not an actual format, just concatenated images aligned to 2048 bytes",
	detect: |file| Certainty::possible_if(file.len() > ALIGNMENT * 2 && !file.starts_with(b"\0\0\x01\xBA") && {
		let mut boundary = ALIGNMENT;
		loop {
			let mut check_buf = [0u8; 8];
			if file.read_chunk_exact(&mut check_buf, boundary).is_err() {
				break false;
			}
			if check_signature(&check_buf) {
				break true;
			}
			boundary += ALIGNMENT;
		}
	}),
	decode: |file| {
		let mut cur_entry_start = 0;
		let mut boundary = ALIGNMENT;
		let mut entries = Vec::new();
		loop {
			let mut check_buf = [0u8; 8];
			if file.read_chunk_exact(&mut check_buf, boundary).is_err() {
				let name = entries.len().to_string();
				entries.push(ArchiveEntry {
					name: name.clone(),
					data: file.subfile(cur_entry_start, file.len() - cur_entry_start).unwrap(),
					timestamp: None
				});
				if entries.len() > 1 {
					return Ok(Archive {format: "concat2k", entries: entries.into()});
				} else {
					return Err("could not find multiple entries".into());
				}
			}
			if check_signature(&check_buf) {
				let name = entries.len().to_string();
				entries.push(ArchiveEntry {
					name: name.clone(),
					data: file.subfile(cur_entry_start, boundary - cur_entry_start).unwrap(),
					timestamp: None
				});
				cur_entry_start = boundary;
			}
			boundary += ALIGNMENT;
		}
	}
};

fn check_signature(buf: &[u8]) -> bool {
	for sig in [b"ogdt", b"TIM2"] {
		if buf.starts_with(sig) || buf[4..].starts_with(sig) {
			return true;
		}
	}
	false
}