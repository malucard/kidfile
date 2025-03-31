use std::sync::LazyLock;
use super::Decoder;

mod lzss;
mod cps;
mod lzss_dc;

pub const DATA_DECODERS: LazyLock<Vec<Decoder<Box<[u8]>>>> = LazyLock::new(|| [
	lzss::ENTRY_LZSS,
	cps::ENTRY_CPS,
	lzss_dc::ENTRY_LZSS_DC
].into());