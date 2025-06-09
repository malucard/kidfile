use std::{fs::File, io::{Read, Seek, SeekFrom}};
use crate::{file_data::FileData, Certainty, Decoder};
use super::{Archive, ArchiveEntry};

// based on code at https://subversion.assembla.com/svn/transprojects/psx/infinity/tools/code/
// no, i have no idea who made that

pub const ENTRY_SLPS02669_DATABIN: Decoder<Archive> = Decoder {
	id: "databin",
	desc: "SLPS-02669 archive",
	detect: |file| {
		if let Some(p) = file.physical_path() {
			if let Some(file_name) = p.file_name().map(|x| x.to_ascii_lowercase()) {
				if file_name == "data.bin" && p.with_file_name("slps_026.69").exists() {
					return Certainty::Certain;
				}
			}
		}
		Certainty::Impossible
	},
	decode: |file| {
		if let Some(data_bin_path) = file.physical_path() {
			if let Some(file_name) = data_bin_path.file_name().map(|x| x.to_ascii_lowercase()) {
				if file_name == "data.bin" {
					if let Ok(mut slps) = File::open(data_bin_path.with_file_name("slps_026.69")) {
						slps.seek( SeekFrom::Start(0x523E8)).map_err(|_| "error while reading slps_026.69")?;
						let mut entry_name = [0u8; 255];
						let mut entries = Vec::new();
						for _ in 0..0xEFC {
							let mut name_pos = 0u32;
							let mut sector = 0u32;
							let mut size = 0u32;
							slps.read_exact(bytemuck::bytes_of_mut(&mut name_pos)).map_err(|_| "error while reading slps_026.69")?;
							slps.read_exact(bytemuck::bytes_of_mut(&mut sector)).map_err(|_| "error while reading slps_026.69")?;
							slps.read_exact(bytemuck::bytes_of_mut(&mut size)).map_err(|_| "error while reading slps_026.69")?;

							let pos_bak = slps.stream_position().map_err(|_| "error while reading slps_026.69")?;
							slps.seek(SeekFrom::Start(name_pos as u64 - 0x8000F800)).map_err(|_| "error while reading slps_026.69")?;
							slps.read_exact(&mut entry_name).map_err(|_| "error while reading slps_026.69")?;
							slps.seek(SeekFrom::Start(pos_bak)).map_err(|_| "error while reading slps_026.69")?;

							entries.push(ArchiveEntry {
								name: std::str::from_utf8(&entry_name).map_err(|_| "error while reading entry name from data.bin")?.into(),
								data: FileData::Stream {path: data_bin_path.clone(), file: None, start: sector as usize * 2048, size: size as usize},
								timestamp: None
							});
						}
						return Ok(Archive {
							format: "databin",
							entries: entries.into()
						});
					} else {
						return Err("data.bin must be accompanied by slps_026.69".into());
					}
				}
			}
		}
		Err("not a data.bin file".into())
	}
};