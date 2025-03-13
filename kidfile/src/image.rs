use std::fmt::Display;
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
#[derive(Zeroable, Pod, Clone, Copy)]
pub struct Pixel {
	r: u8,
	g: u8,
	b: u8,
	a: u8
}

pub struct Frame {
	pub width: u32,
	pub height: u32,
	pub og_fmt: PixelFormat,
	pub pixels: Box<[Pixel]>
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

	pub fn paste(&mut self, x: u32, y: u32, o: &Frame) {
		assert!(x + o.width <= self.width && y + o.height <= self.height);
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