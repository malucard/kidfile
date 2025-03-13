use file_data::FileData;
use image::Image;

pub mod file_data;
pub mod byte_slice;
pub mod image;
mod data_formats;
pub use data_formats::DATA_DECODERS;
mod archive_formats;
pub use archive_formats::{Archive, ARCHIVE_DECODERS};
mod image_formats;
pub use image_formats::IMAGE_DECODERS;

pub enum Certainty {
	Impossible,
	Possible,
	Certain
}

impl Certainty {
	pub const fn certain_if(cond: bool) -> Self {
		if cond {
			Self::Certain
		} else {
			Self::Impossible
		}
	}

	pub const fn possible_if(cond: bool) -> Self {
		if cond {
			Self::Possible
		} else {
			Self::Impossible
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Decoder<T> {
	id: &'static str,
	desc: &'static str,
	detect: fn(data: &mut FileData) -> Certainty,
	decode: fn(data: &mut FileData) -> Result<T, String>
}

pub enum DynData {
	Raw(FileData),
	Archive(Archive),
	Image(Image)
}

impl From<FileData> for DynData {
	fn from(value: FileData) -> Self {
		Self::Raw(value)
	}
}

impl From<Box<[u8]>> for DynData {
	fn from(value: Box<[u8]>) -> Self {
		Self::Raw(FileData::Memory {buf: value})
	}
}

impl From<Archive> for DynData {
	fn from(value: Archive) -> Self {
		Self::Archive(value)
	}
}

impl From<Image> for DynData {
	fn from(value: Image) -> Self {
		Self::Image(value)
	}
}

fn decode_step<T: Into<DynData>>(data: &mut FileData, decoders: &[Decoder<T>], disallow_id: Option<&'static str>, discard_low_confidence: bool) -> Result<Option<(&'static str, DynData)>, String> {
	for decoder in decoders {
		if let Certainty::Certain = (decoder.detect)(data) {
			if Some(decoder.id) == disallow_id {
				return Ok(None);
			}
			return (decoder.decode)(data)
				.map(|x| Some((decoder.id, x.into())))
				.map_err(|msg| {
					if msg.is_empty() {
						format!("unspecified error from {}", decoder.id)
					} else {
						format!("error from {}: {}", decoder.id, msg)
					}
				});
		}
	}
	if !discard_low_confidence {
		for decoder in decoders {
			if Some(decoder.id) != disallow_id {
				if let Certainty::Possible = (decoder.detect)(data) {
					if let Ok(x) = (decoder.decode)(data) {
						return Ok(Some((decoder.id, x.into())));
					}
				}
			}
		}
	}
	Ok(None)
}

pub fn auto_decode_step(data: &mut FileData, disallow_id: Option<&'static str>, in_archive: Option<&'static str>) -> Result<(&'static str, DynData), String> {
	if let Some(x) = decode_step(data, &ARCHIVE_DECODERS, disallow_id, in_archive.is_some())? {
		return Ok(x);
	}
	if let Some(x) = decode_step(data, &IMAGE_DECODERS, disallow_id, false)? {
		return Ok(x);
	}
	if let Some(x) = decode_step(data, &DATA_DECODERS, disallow_id, false)? {
		return Ok(x);
	}
	Err("could not fully decode file".into())
}

pub struct DecodeResult {
	pub data: DynData,
	pub steps_taken: Vec<&'static str>,
	pub error_msg: String
}

pub fn auto_decode_full(initial_data: &mut FileData, in_archive: Option<&'static str>) -> DecodeResult {
	let mut steps_taken = Vec::<&'static str>::new();
	let mut cur_data = None;
	loop {
		match auto_decode_step(cur_data.as_mut().unwrap_or(initial_data), steps_taken.last().cloned(), in_archive) {
			Ok((id, decoded)) => {
				steps_taken.push(id);
				if let DynData::Raw(new_data) = decoded {
					cur_data = Some(new_data);
				} else {
					return DecodeResult {
						data: decoded,
						steps_taken,
						error_msg: String::new()
					}
				}
			}
			Err(msg) => {
				return DecodeResult {
					data: DynData::Raw(cur_data.unwrap_or_else(|| initial_data.clone())),
					steps_taken,
					error_msg: msg
				}
			}
		}
	}
}