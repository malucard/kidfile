use std::sync::LazyLock;
use crate::image::Image;
use super::Decoder;

mod tim2;
mod ogdt;
mod gim;
mod klz;
mod bip;
mod pvr;
mod common_image;

pub const IMAGE_DECODERS: LazyLock<Vec<Decoder<Image>>> = LazyLock::new(|| [
	tim2::ENTRY_TIM2,
	ogdt::ENTRY_OGDT,
	gim::ENTRY_GIM,
	klz::ENTRY_KLZ,
	bip::ENTRY_BIP,
	pvr::ENTRY_PVR,
	common_image::ENTRY_PNG,
	common_image::ENTRY_JPEG,
	common_image::ENTRY_BMP,
	common_image::ENTRY_GIF
].into());