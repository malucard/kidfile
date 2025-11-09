use crate::{image::{Frame, Image, PixelFormat}, Certainty, Decoder};

pub const ENTRY_TIM2: Decoder<Image> = Decoder {
	id: "tim2",
	desc: "PlayStation 2 official image format",
	detect: |file| Certainty::certain_if(file.starts_with(b"TIM2") && !file.starts_with_at(b"PNGFILE3", 0x40)),
	decode: |file| {
		let tim2_img = tim2::from_buffer(file.read()).map_err(|e| format!("{:?}", e))?;
		let frames = tim2_img.frames().iter().filter_map(|tim2_frame| {
			let pixels = tim2_frame.to_raw(None);
			if pixels.is_empty() {
				None
			} else {
				let mut frame = Frame::from_rgba(tim2_frame.width() as u32, tim2_frame.height() as u32, &pixels);
				frame.og_fmt = match tim2_frame.format().unwrap() {
					tim2::Format::Indexed4 => PixelFormat::RgbaClut4,
					tim2::Format::Indexed8 => PixelFormat::RgbaClut8,
					tim2::Format::Rgb888 => PixelFormat::Rgb,
					tim2::Format::Rgba8888 => PixelFormat::Rgba,
					tim2::Format::Abgr1555 => PixelFormat::Rgba5551
				};
				if matches!(frame.og_fmt, PixelFormat::RgbaClut4 | PixelFormat::RgbaClut8 | PixelFormat::Rgba) {
					Some(frame.with_double_alpha())
				} else {
					Some(frame)
				}
			}
		}).collect::<Box<[_]>>();
		if frames.is_empty() {
			Err("no frames were decoded successfully".into())
		} else {
			Ok(Image {frames})
		}
	}
};
