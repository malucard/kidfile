use std::{fs::File, io::{BufReader, Read, Seek, SeekFrom}, path::PathBuf};

pub enum FileData {
	Memory {
		buf: Box<[u8]>
	},
	MemoryCompressed {
		buf: Box<[u8]>,
		full_size: usize,
		decompress: fn(Box<[u8]>, usize) -> Box<[u8]>
	},
	Stream {
		path: PathBuf,
		file: Option<BufReader<File>>,
		start: usize,
		size: usize
	},
	StreamCompressed {
		path: PathBuf,
		file: Option<BufReader<File>>,
		start: usize,
		size: usize,
		full_size: usize,
		decompress: fn(Box<[u8]>, usize) -> Box<[u8]>
	}
}

impl FileData {
	pub fn len(&self) -> usize {
		match self {
			Self::Memory {buf} => buf.len(),
			Self::MemoryCompressed {full_size, ..} => *full_size,
			Self::Stream {size, ..} => *size,
			Self::StreamCompressed {full_size, ..} => *full_size
		}
	}

	pub fn subfile(&mut self, sub_start: usize, sub_size: usize) -> Result<FileData, ()> {
		match self {
			Self::Stream {path, start, size, ..} => {
				if sub_start + sub_size > *size {
					return Err(());
				}
				Ok(Self::Stream {
					path: path.clone(),
					file: None,
					start: *start + sub_start,
					size: sub_size
				})
			}
			_ => {
				let mut buf = unsafe {Box::new_uninit_slice(sub_size).assume_init()};
				self.read_chunk_exact(&mut buf, sub_start)?;
				Ok(Self::Memory {buf})
			}
		}
	}

	pub fn starts_with_at(&mut self, needle: &[u8], offset: usize) -> bool {
		match self {
			Self::Stream {path, file, start, size, ..} => {
				if offset + needle.len() > *size {
					return false;
				}
				if file.is_none() {
					*file = Some(BufReader::new(File::open(path).unwrap()));
				}
				let file = file.as_mut().unwrap();
				if file.seek(SeekFrom::Start((*start + offset) as u64)).is_err() {
					return false;
				}
				let mut sig = vec![0u8; needle.len()];
				file.read_exact(&mut sig).is_ok() && sig == needle
			}
			_ => self.read().get(offset..).map_or(false, |x| x.starts_with(needle))
		}
	}

	pub fn starts_with(&mut self, needle: &[u8]) -> bool {
		self.starts_with_at(needle, 0)
	}

	pub fn read_chunk_exact(&mut self, out_buf: &mut [u8], chunk_start: usize) -> Result<(), ()> {
		match self {
			Self::Memory {..} => {}
			Self::MemoryCompressed {buf, full_size, decompress} => {
				if chunk_start + out_buf.len() > *full_size {
					return Err(());
				}
				*self = Self::Memory {buf: decompress(std::mem::take(buf), *full_size)};
			}
			Self::Stream {path, file, start, size} => {
				if chunk_start + out_buf.len() > *size {
					return Err(());
				}
				if file.is_none() {
					*file = Some(BufReader::new(File::open(path).unwrap()));
				}
				let file = file.as_mut().unwrap();
				file.seek(SeekFrom::Start((*start + chunk_start) as u64)).map_err(|_| ())?;
				file.read_exact(out_buf).map_err(|_| ())?;
				return Ok(());
			}
			Self::StreamCompressed {path, file, start, size, full_size, decompress} => {
				if chunk_start + out_buf.len() > *full_size {
					return Err(());
				}
				if file.is_none() {
					*file = Some(BufReader::new(File::open(path).unwrap()));
				}
				let file = file.as_mut().unwrap();
				file.seek(SeekFrom::Start(*start as u64)).map_err(|_| ())?;
				let mut compressed = unsafe {Box::new_uninit_slice(*size).assume_init()};
				file.read_exact(&mut compressed).map_err(|_| ())?;
				*self = Self::Memory {buf: decompress(compressed, *full_size)};
			}
		}
		match self {
			Self::Memory {buf} => out_buf.copy_from_slice(&buf.get(chunk_start..chunk_start + out_buf.len()).ok_or(())?),
			_ => unreachable!()
		}
		Ok(())
	}

	pub fn read(&mut self) -> &[u8] {
		match self {
			Self::Memory {buf} => return buf,
			Self::MemoryCompressed {buf, full_size, decompress} => {
				*self = Self::Memory {buf: decompress(std::mem::take(buf), *full_size)};
			}
			Self::Stream {path, file, start, size} => {
				if file.is_none() {
					*file = Some(BufReader::new(File::open(path).unwrap()));
				}
				let file = file.as_mut().unwrap();
				file.seek(SeekFrom::Start(*start as u64)).unwrap();
				let mut buf = unsafe {Box::new_uninit_slice(*size).assume_init()};
				file.read_exact(&mut buf).unwrap();
				*self = Self::Memory {buf};
			}
			Self::StreamCompressed {path, file, start, size, full_size, decompress} => {
				if file.is_none() {
					*file = Some(BufReader::new(File::open(path).unwrap()));
				}
				let file = file.as_mut().unwrap();
				file.seek(SeekFrom::Start(*start as u64)).unwrap();
				let mut compressed = unsafe {Box::new_uninit_slice(*size).assume_init()};
				file.read_exact(&mut compressed).unwrap();
				*self = Self::Memory {buf: decompress(compressed, *full_size)};
			}
		}
		match self {
			Self::Memory {buf} => buf,
			_ => unreachable!()
		}
	}

	pub fn get_u8_at(&mut self, offset: usize) -> Result<u8, ()> {
		let mut bytes = [0u8; 1];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(u8::from_le_bytes(bytes))
	}
	pub fn get_u16_at(&mut self, offset: usize) -> Result<u16, ()> {
		let mut bytes = [0u8; 2];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(u16::from_le_bytes(bytes))
	}
	pub fn get_u32_at(&mut self, offset: usize) -> Result<u32, ()> {
		let mut bytes = [0u8; 4];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(u32::from_le_bytes(bytes))
	}
	pub fn get_u64_at(&mut self, offset: usize) -> Result<u64, ()> {
		let mut bytes = [0u8; 8];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(u64::from_le_bytes(bytes))
	}
	pub fn get_i8_at(&mut self, offset: usize) -> Result<i8, ()> {
		let mut bytes = [0u8; 1];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(i8::from_le_bytes(bytes))
	}
	pub fn get_i16_at(&mut self, offset: usize) -> Result<i16, ()> {
		let mut bytes = [0u8; 2];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(i16::from_le_bytes(bytes))
	}
	pub fn get_i32_at(&mut self, offset: usize) -> Result<i32, ()> {
		let mut bytes = [0u8; 4];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(i32::from_le_bytes(bytes))
	}
	pub fn get_i64_at(&mut self, offset: usize) -> Result<i64, ()> {
		let mut bytes = [0u8; 8];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(i64::from_le_bytes(bytes))
	}
	pub fn get_u8_be_at(&mut self, offset: usize) -> Result<u8, ()> {
		let mut bytes = [0u8; 1];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(u8::from_be_bytes(bytes))
	}
	pub fn get_u16_be_at(&mut self, offset: usize) -> Result<u16, ()> {
		let mut bytes = [0u8; 2];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(u16::from_be_bytes(bytes))
	}
	pub fn get_u32_be_at(&mut self, offset: usize) -> Result<u32, ()> {
		let mut bytes = [0u8; 4];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(u32::from_be_bytes(bytes))
	}
	pub fn get_u64_be_at(&mut self, offset: usize) -> Result<u64, ()> {
		let mut bytes = [0u8; 8];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(u64::from_be_bytes(bytes))
	}
	pub fn get_i8_be_at(&mut self, offset: usize) -> Result<i8, ()> {
		let mut bytes = [0u8; 1];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(i8::from_be_bytes(bytes))
	}
	pub fn get_i16_be_at(&mut self, offset: usize) -> Result<i16, ()> {
		let mut bytes = [0u8; 2];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(i16::from_be_bytes(bytes))
	}
	pub fn get_i32_be_at(&mut self, offset: usize) -> Result<i32, ()> {
		let mut bytes = [0u8; 4];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(i32::from_be_bytes(bytes))
	}
	pub fn get_i64_be_at(&mut self, offset: usize) -> Result<i64, ()> {
		let mut bytes = [0u8; 8];
		self.read_chunk_exact(&mut bytes, offset)?;
		Ok(i64::from_be_bytes(bytes))
	}
}

impl Clone for FileData {
	fn clone(&self) -> Self {
		match self {
			Self::Memory {buf} => Self::Memory {buf: buf.clone()},
			Self::MemoryCompressed {buf, decompress, full_size} => Self::MemoryCompressed {
				buf: buf.clone(),
				decompress: *decompress,
				full_size: *full_size
			},
			Self::Stream {path, start, size, ..} => Self::Stream {
				path: path.clone(),
				file: None,
				start: *start,
				size: *size
			},
			Self::StreamCompressed {path, start, size, decompress, full_size, ..} => Self::StreamCompressed {
				path: path.clone(),
				file: None,
				start: *start,
				size: *size,
				decompress: *decompress,
				full_size: *full_size
			}
		}
	}
}