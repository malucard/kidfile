use std::sync::LazyLock;
use crate::file_data::FileData;
use super::Decoder;

mod afs;
mod lnk;
mod concat2k;

pub struct ArchiveEntry {
	pub data: FileData,
	pub name: String,
	pub timestamp: Option<(u16, u16, u16, u16, u16, u16)>
}

pub struct Archive {
	pub format: &'static str,
	pub entries: Box<[ArchiveEntry]>
}

pub const ARCHIVE_DECODERS: LazyLock<Vec<Decoder<Archive>>> = LazyLock::new(|| [
	afs::ENTRY_AFS,
	lnk::ENTRY_LNK,
	concat2k::ENTRY_CONCAT2K
].into());