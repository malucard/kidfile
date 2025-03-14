use std::sync::LazyLock;
use crate::image::Image;
use super::Decoder;

mod common_image;
mod tim2;
mod klz;
mod ogdt;
mod gim;

pub const IMAGE_DECODERS: LazyLock<Vec<Decoder<Image>>> = LazyLock::new(|| [
	klz::ENTRY_KLZ,
	tim2::ENTRY_TIM2,
	ogdt::ENTRY_OGDT,
	//gim::ENTRY_GIM,
	common_image::ENTRY_PNG,
	common_image::ENTRY_JPEG,
	common_image::ENTRY_BMP,
	common_image::ENTRY_GIF
].into());