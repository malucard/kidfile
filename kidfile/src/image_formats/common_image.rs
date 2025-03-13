use crate::{file_data::FileData, image::{Frame, Image}, Certainty, Decoder};

pub const ENTRY_PNG: Decoder<Image> = Decoder {
	id: "png",
	desc: "PNG",
	detect: |buf| Certainty::certain_if(buf.starts_with(b"\x89PNG\x0d\x0a\x1a\x0a")),
	decode
};

pub const ENTRY_JPEG: Decoder<Image> = Decoder {
	id: "jpeg",
	desc: "JPEG",
	detect: |buf| Certainty::certain_if(buf.starts_with(&[0xff, 0xd8, 0xff])),
	decode
};

pub const ENTRY_BMP: Decoder<Image> = Decoder {
	id: "bmp",
	desc: "BMP",
	detect: |buf| Certainty::certain_if(buf.starts_with(b"BM")),
	decode
};

pub const ENTRY_GIF: Decoder<Image> = Decoder {
	id: "gif",
	desc: "GIF",
	detect: |buf| Certainty::certain_if(buf.starts_with(b"GIF89a") || buf.starts_with(b"GIF87a")),
	decode
};

fn decode(file: &mut FileData) -> Result<Image, String> {
	let loaded = image::load_from_memory(file.read()).map_err(|e| e.to_string())?;
	Ok(Image {
		frames: Box::new([
			Frame::from_rgba(loaded.width() as u32, loaded.height() as u32, &loaded.to_rgba8())
		])
	})
}