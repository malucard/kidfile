use std::{fmt::Display, mem::MaybeUninit};
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
	Rgba,
	Rgbx,
	Rgb,
	Bgra,
	Bgrx,
	Bgr,
	Rgba16,
	Rgb16,
	Rgba4444,
	Bgra16,
	Bgr16,
	Bgra4444,
	RgbaClut8,
	RgbxClut8,
	RgbClut8,
	BgraClut8,
	BgrxClut8,
	BgrClut8,
	RgbaClut4,
	RgbxClut4,
	RgbClut4,
	BgraClut4,
	BgrxClut4,
	BgrClut4
}

impl PixelFormat {
	pub fn bpp(self) -> usize {
		match self {
			PixelFormat::Rgba => 32,
			PixelFormat::Rgbx => 32,
			PixelFormat::Rgb => 24,
			PixelFormat::Bgra => 32,
			PixelFormat::Bgrx => 32,
			PixelFormat::Bgr => 24,
			PixelFormat::Rgba16 => 16,
			PixelFormat::Rgb16 => 16,
			PixelFormat::Rgba4444 => 16,
			PixelFormat::Bgra16 => 16,
			PixelFormat::Bgr16 => 16,
			PixelFormat::Bgra4444 => 16,
			PixelFormat::RgbaClut8 => 8,
			PixelFormat::RgbxClut8 => 8,
			PixelFormat::RgbClut8 => 8,
			PixelFormat::BgraClut8 => 8,
			PixelFormat::BgrxClut8 => 8,
			PixelFormat::BgrClut8 => 8,
			PixelFormat::RgbaClut4 => 4,
			PixelFormat::RgbxClut4 => 4,
			PixelFormat::RgbClut4 => 4,
			PixelFormat::BgraClut4 => 4,
			PixelFormat::BgrxClut4 => 4,
			PixelFormat::BgrClut4 => 4
		}
	}
}

impl<'a> From<&png::Info<'a>> for PixelFormat {
	fn from(info: &png::Info) -> Self {
		match (info.color_type, info.bit_depth) {
			(png::ColorType::Indexed, png::BitDepth::Four) => PixelFormat::BgraClut4,
			(png::ColorType::Indexed, png::BitDepth::Eight) => PixelFormat::BgraClut8,
			(png::ColorType::Rgb, _) => PixelFormat::Bgr,
			(png::ColorType::Rgba, _) => PixelFormat::Bgra,
			_ => unreachable!()
		}
	}
}

impl Display for PixelFormat {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Rgba => write!(f, "RGBA"),
			Self::Rgbx => write!(f, "RGBX"),
			Self::Rgb => write!(f, "RGB"),
			Self::Bgra => write!(f, "BGRA"),
			Self::Bgrx => write!(f, "BGRX"),
			Self::Bgr => write!(f, "BGR"),
			Self::Rgba16 => write!(f, "RGBA5551"),
			Self::Rgb16 => write!(f, "RGB565"),
			Self::Rgba4444 => write!(f, "RGBA4444"),
			Self::Bgra16 => write!(f, "BGRA5551"),
			Self::Bgr16 => write!(f, "BGR565"),
			Self::Bgra4444 => write!(f, "BGRA4444"),
			Self::RgbaClut8 => write!(f, "RGBA clut8"),
			Self::RgbxClut8 => write!(f, "RGBX clut8"),
			Self::RgbClut8 => write!(f, "RGB clut8"),
			Self::BgraClut8 => write!(f, "BGRA clut8"),
			Self::BgrxClut8 => write!(f, "BGRX clut8"),
			Self::BgrClut8 => write!(f, "BGR clut8"),
			Self::RgbaClut4 => write!(f, "RGBA clut4"),
			Self::RgbxClut4 => write!(f, "RGBX clut4"),
			Self::RgbClut4 => write!(f, "RGB clut4"),
			Self::BgraClut4 => write!(f, "BGRA clut4"),
			Self::BgrxClut4 => write!(f, "BGRX clut4"),
			Self::BgrClut4 => write!(f, "BGR clut4")
		}
	}
}

#[repr(C)]
#[derive(Zeroable, Pod, Clone, Copy, Default)]
pub struct Pixel {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub a: u8
}

pub struct Frame {
	pub width: u32,
	pub height: u32,
	pub og_fmt: PixelFormat,
	pub pixels: Box<[Pixel]>
}

fn bits_4_to_8(x: u8) -> u8 {
	x << 4 | (x & 0xF)
}

fn bits_5_to_8(x: u8) -> u8 {
	x << 3 | (x & 1) << 2 | (x & 1) << 1 | (x & 1)
}

fn bits_6_to_8(x: u8) -> u8 {
	x << 2 | (x & 1) << 1 | (x & 1)
}

impl Frame {
	pub fn empty(width: u32, height: u32, og_fmt: PixelFormat) -> Self {
		Self {
			width, height, og_fmt,
			pixels: vec![Pixel {r: 0, g: 0, b: 0, a: 0}; (width * height) as usize].into()
		}
	}

	pub fn from_rgba(width: u32, height: u32, buf: &[u8]) -> Self {
		let needed_size = width * height * 4;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::Rgba,
			pixels: bytemuck::cast_slice(buf).into()
		}
	}

	pub fn from_rgba16(width: u32, height: u32, buf: &[u8]) -> Self {
		let needed_size = width * height * 2;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::Rgba16,
			pixels: buf.chunks_exact(2).map(|x| Pixel {
				r: bits_5_to_8(x[0]),
				g: bits_5_to_8(x[0] >> 5 | x[1] << 3),
				b: bits_5_to_8(x[1] >> 3),
				a: if x[1] & 0x80 != 0 {0xFF} else {0}
			}).collect()
		}
	}

	pub fn from_bgra16(width: u32, height: u32, buf: &[u8]) -> Self {
		let needed_size = width * height * 2;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::Bgra16,
			pixels: buf.chunks_exact(2).map(|x| Pixel {
				r: bits_5_to_8(x[1] >> 3),
				g: bits_5_to_8(x[0] >> 5 | x[1] << 3),
				b: bits_5_to_8(x[0]),
				a: if x[1] & 0x80 != 0 {0xFF} else {0}
			}).collect()
		}
	}

	pub fn from_rgba4444(width: u32, height: u32, buf: &[u8]) -> Self {
		let needed_size = width * height * 2;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::Rgba4444,
			pixels: buf.chunks_exact(2).map(|x| Pixel {
				r: bits_4_to_8(x[0]),
				g: bits_4_to_8(x[0] >> 4),
				b: bits_4_to_8(x[1]),
				a: bits_4_to_8(x[1] >> 4)
			}).collect()
		}
	}

	pub fn from_bgra4444(width: u32, height: u32, buf: &[u8]) -> Self {
		let needed_size = width * height * 2;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::Bgra4444,
			pixels: buf.chunks_exact(2).map(|x| Pixel {
				r: bits_4_to_8(x[1]),
				g: bits_4_to_8(x[0] >> 4),
				b: bits_4_to_8(x[0]),
				a: bits_4_to_8(x[1] >> 4)
			}).collect()
		}
	}

	pub fn from_rgb16(width: u32, height: u32, buf: &[u8]) -> Self {
		let needed_size = width * height * 2;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::Rgb16,
			pixels: buf.chunks_exact(2).map(|x| Pixel {
				r: bits_5_to_8(x[0]),
				g: bits_6_to_8(x[0] >> 5 | x[1] << 3),
				b: bits_5_to_8(x[1] >> 3),
				a: 0xFF
			}).collect()
		}
	}

	pub fn from_bgr16(width: u32, height: u32, buf: &[u8]) -> Self {
		let needed_size = width * height * 2;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::Bgr16,
			pixels: buf.chunks_exact(2).map(|x| Pixel {
				r: bits_5_to_8(x[1] >> 3),
				g: bits_6_to_8(x[0] >> 5 | x[1] << 3),
				b: bits_5_to_8(x[0]),
				a: 0xFF
			}).collect()
		}
	}

	pub fn from_rgba_clut8(width: u32, height: u32, clut: &[u8], buf: &[u8]) -> Self {
		let needed_size = width * height;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::RgbaClut8,
			pixels: buf.iter().cloned().map(|x| Pixel {r: clut[x as usize * 4 + 0], g: clut[x as usize * 4 + 1], b: clut[x as usize * 4 + 2], a: clut[x as usize * 4 + 3]}).collect()
		}
	}

	pub fn from_rgba_clut4(width: u32, height: u32, clut: &[u8], buf: &[u8]) -> Self {
		let needed_size = width * height / 2;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::RgbaClut4,
			pixels: buf.iter().cloned().map(|x| [
				Pixel {r: clut[(x as usize & 0xF) * 4 + 0], g: clut[(x as usize & 0xF) * 4 + 1], b: clut[(x as usize & 0xF) * 4 + 2], a: clut[(x as usize & 0xF) * 4 + 3]},
				Pixel {r: clut[(x as usize >> 4) * 4 + 0], g: clut[(x as usize >> 4) * 4 + 1], b: clut[(x as usize >> 4) * 4 + 2], a: clut[(x as usize >> 4) * 4 + 3]}
			]).flatten().collect()
		}
	}

	pub fn from_bgra(width: u32, height: u32, buf: &[u8]) -> Self {
		let needed_size = width * height * 4;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::Bgra,
			pixels: buf.chunks_exact(4).map(|x| Pixel {r: x[2], g: x[1], b: x[0], a: x[3]}).collect()
		}
	}

	pub fn from_bgra_clut8(width: u32, height: u32, clut: &[u8], buf: &[u8]) -> Self {
		let needed_size = width * height;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::BgraClut8,
			pixels: buf.iter().cloned().map(|x| Pixel {r: clut[x as usize * 4 + 2], g: clut[x as usize * 4 + 1], b: clut[x as usize * 4 + 0], a: clut[x as usize * 4 + 3]}).collect()
		}
	}

	pub fn from_bgra_clut4(width: u32, height: u32, clut: &[u8], buf: &[u8]) -> Self {
		let needed_size = width * height / 2;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::BgraClut4,
			pixels: buf.iter().cloned().map(|x| [
				Pixel {r: clut[(x as usize & 0xF) * 4 + 2], g: clut[(x as usize & 0xF) * 4 + 1], b: clut[(x as usize & 0xF) * 4 + 0], a: clut[(x as usize & 0xF) * 4 + 3]},
				Pixel {r: clut[(x as usize >> 4) * 4 + 2], g: clut[(x as usize >> 4) * 4 + 1], b: clut[(x as usize >> 4) * 4 + 0], a: clut[(x as usize >> 4) * 4 + 3]}
			]).flatten().collect()
		}
	}

	pub fn from_rgbx(width: u32, height: u32, buf: &[u8]) -> Self {
		let needed_size = width * height * 4;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::Rgbx,
			pixels: buf.chunks_exact(4).map(|x| Pixel {r: x[0], g: x[1], b: x[2], a: 255}).collect()
		}
	}

	pub fn from_rgbx_clut8(width: u32, height: u32, clut: &[u8], buf: &[u8]) -> Self {
		let needed_size = width * height;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::RgbxClut8,
			pixels: buf.iter().cloned().map(|x| Pixel {r: clut[x as usize * 4 + 0], g: clut[x as usize * 4 + 1], b: clut[x as usize * 4 + 2], a: 255}).collect()
		}
	}

	pub fn from_rgbx_clut4(width: u32, height: u32, clut: &[u8], buf: &[u8]) -> Self {
		let needed_size = width * height / 2;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::RgbxClut4,
			pixels: buf.iter().cloned().map(|x| [
				Pixel {r: clut[(x as usize & 0xF) * 4 + 0], g: clut[(x as usize & 0xF) * 4 + 1], b: clut[(x as usize & 0xF) * 4 + 2], a: 255},
				Pixel {r: clut[(x as usize >> 4) * 4 + 0], g: clut[(x as usize >> 4) * 4 + 1], b: clut[(x as usize >> 4) * 4 + 2], a: 255}
			]).flatten().collect()
		}
	}

	pub fn from_bgrx(width: u32, height: u32, buf: &[u8]) -> Self {
		let needed_size = width * height * 4;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::Bgrx,
			pixels: buf.chunks_exact(4).map(|x| Pixel {r: x[2], g: x[1], b: x[0], a: 255}).collect()
		}
	}

	pub fn from_bgrx_clut8(width: u32, height: u32, clut: &[u8], buf: &[u8]) -> Self {
		let needed_size = width * height;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::BgrxClut8,
			pixels: buf.iter().cloned().map(|x| Pixel {r: clut[x as usize * 4 + 2], g: clut[x as usize * 4 + 1], b: clut[x as usize * 4 + 0], a: 255}).collect()
		}
	}

	pub fn from_bgrx_clut4(width: u32, height: u32, clut: &[u8], buf: &[u8]) -> Self {
		let needed_size = width * height / 2;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::BgrxClut4,
			pixels: buf.iter().cloned().map(|x| [
				Pixel {r: clut[(x as usize & 0xF) * 4 + 2], g: clut[(x as usize & 0xF) * 4 + 1], b: clut[(x as usize & 0xF) * 4 + 0], a: 255},
				Pixel {r: clut[(x as usize >> 4) * 4 + 2], g: clut[(x as usize >> 4) * 4 + 1], b: clut[(x as usize >> 4) * 4 + 0], a: 255}
			]).flatten().collect()
		}
	}

	pub fn from_rgb(width: u32, height: u32, buf: &[u8]) -> Self {
		let needed_size = width * height * 3;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::Rgb,
			pixels: buf.chunks_exact(3).map(|x| Pixel {r: x[0], g: x[1], b: x[2], a: 255}).collect()
		}
	}

	pub fn from_rgb_clut8(width: u32, height: u32, clut: &[u8], buf: &[u8]) -> Self {
		let needed_size = width * height;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::RgbClut8,
			pixels: buf.iter().cloned().map(|x| Pixel {r: clut[x as usize * 3 + 0], g: clut[x as usize * 3 + 1], b: clut[x as usize * 3 + 2], a: 255}).collect()
		}
	}

	pub fn from_rgb_clut4(width: u32, height: u32, clut: &[u8], buf: &[u8]) -> Self {
		let needed_size = width * height / 2;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::RgbClut4,
			pixels: buf.iter().cloned().map(|x| [
				Pixel {r: clut[(x as usize & 0xF) * 3 + 0], g: clut[(x as usize & 0xF) * 3 + 1], b: clut[(x as usize & 0xF) * 3 + 2], a: 255},
				Pixel {r: clut[(x as usize >> 4) * 3 + 0], g: clut[(x as usize >> 4) * 3 + 1], b: clut[(x as usize >> 4) * 3 + 2], a: 255}
			]).flatten().collect()
		}
	}

	pub fn from_bgr(width: u32, height: u32, buf: &[u8]) -> Self {
		let needed_size = width * height * 3;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::Bgr,
			pixels: buf.chunks_exact(3).map(|x| Pixel {r: x[2], g: x[1], b: x[0], a: 255}).collect()
		}
	}

	pub fn from_bgr_clut8(width: u32, height: u32, clut: &[u8], buf: &[u8]) -> Self {
		let needed_size = width * height;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::BgrClut8,
			pixels: buf.iter().cloned().map(|x| Pixel {r: clut[x as usize * 3 + 2], g: clut[x as usize * 3 + 1], b: clut[x as usize * 3 + 0], a: 255}).collect()
		}
	}

	pub fn from_bgr_clut4(width: u32, height: u32, clut: &[u8], buf: &[u8]) -> Self {
		let needed_size = width * height / 2;
		assert!(buf.len() as u32 >= needed_size);
		let buf = &buf[0..needed_size as usize];
		Self {
			width, height, og_fmt: PixelFormat::BgrClut4,
			pixels: buf.iter().cloned().map(|x| [
				Pixel {r: clut[(x as usize & 0xF) * 3 + 2], g: clut[(x as usize & 0xF) * 3 + 1], b: clut[(x as usize & 0xF) * 3 + 0], a: 255},
				Pixel {r: clut[(x as usize >> 4) * 3 + 2], g: clut[(x as usize >> 4) * 3 + 1], b: clut[(x as usize >> 4) * 3 + 0], a: 255}
			]).flatten().collect()
		}
	}

	pub fn with_og_fmt(mut self, og_fmt: PixelFormat) -> Self {
		self.og_fmt = og_fmt;
		self
	}

	pub fn with_double_alpha(mut self) -> Self {
		for p in &mut self.pixels {
			p.a = (p.a as u16 * 255 / 128).min(255) as u8;
		}
		self
	}

	pub fn row(&self, y: u32) -> &[Pixel] {
		let w = self.width as usize;
		&self.pixels[y as usize * w..y as usize * w + w]
	}

	pub fn row_mut(&mut self, y: u32) -> &mut [Pixel] {
		let w = self.width as usize;
		&mut self.pixels[y as usize * w..y as usize * w + w]
	}

	pub fn resize(&mut self, w: u32, h: u32) {
		let mut pixels = if w == self.width {
			self.pixels.to_vec()
		} else {
			let mut pixels = Vec::with_capacity((w * h) as usize);
			let common_w = self.width.min(w);
			let common_h = self.height.min(h);
			for y in 0..common_h {
				for x in 0..common_w {
					pixels.push(self.pixels[(y * self.width + x) as usize]);
				}
				for _ in common_w..w {
					pixels.push(Pixel::zeroed());
				}
			}
			pixels
		};
		pixels.resize((w * h) as usize, Pixel::zeroed());
		self.pixels = pixels.into_boxed_slice();
		self.width = w;
		self.height = h;
	}

	pub fn resized(mut self, w: u32, h: u32) -> Self {
		self.resize(w, h);
		self
	}

	pub fn crushed_down(mut self, w: u32, h: u32) -> Self {
		if w == self.width && h == self.height {
			return self;
		}
		let mut pixels = self.pixels.into_vec();
		let xchunks = self.width / 32;
		let ychunks = self.height / 32;
		let dst_chunk_width = 32 * w / self.width;
		let dst_chunk_height = 32 * h / self.height;
		let mut dst_idx = 0;
		for chunk_y_idx in 0..ychunks {
			for sub_y in 0..dst_chunk_height {
				for chunk_x_idx in 0..xchunks {
					for sub_x in 0..dst_chunk_width {
						pixels[dst_idx] = pixels[((chunk_y_idx * 32 + sub_y) * self.width + chunk_x_idx * 32 + sub_x) as usize];
						dst_idx += 1;
					}
				}
			}
		}
		pixels.resize((w * h) as usize, Pixel::zeroed());
		self.pixels = pixels.into();
		self.width = w;
		self.height = h;
		self
	}

	pub fn paste(&mut self, x: u32, y: u32, o: &Frame) {
		let end_x = (x + o.width).min(self.width);
		let end_y = (y + o.height).min(self.height);
		if end_x > x && end_y > y {
			for row_y in y..end_y {
				self.row_mut(row_y)[x as usize..end_x as usize].copy_from_slice(&o.row(row_y - y)[..(end_x - x) as usize]);
			}
		}
	}

	pub fn paste_resizing(&mut self, x: u32, y: u32, o: &Frame) {
		if x + o.width > self.width || y + o.height > self.height {
			self.resize(self.width.max(x + o.width), self.height.max(y + o.height));
		}
		for oy in 0..o.height {
			self.row_mut(y + oy)[x as usize..(x + o.width) as usize].copy_from_slice(o.row(oy));
		}
	}

	pub fn as_rgba_bytes(&self) -> &[u8] {
		bytemuck::cast_slice(&self.pixels)
	}
}

pub struct Image {
	pub frames: Box<[Frame]>
}