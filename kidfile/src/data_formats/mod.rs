use std::sync::LazyLock;
use super::Decoder;

mod lzss;
mod cps;
mod lzss_be;

pub const DATA_DECODERS: LazyLock<Vec<Decoder<Box<[u8]>>>> = LazyLock::new(|| [
	lzss::ENTRY_LZSS,
	cps::ENTRY_CPS,
	lzss_be::ENTRY_LZSS_BE
].into());