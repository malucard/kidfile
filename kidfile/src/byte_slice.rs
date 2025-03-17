macro_rules! impl_byte_readers {
	($($t:ty),*) => {paste::paste! {$(
		fn [<read_ $t>](&self, offset: usize) -> Result<$t, String> {
			Ok($t::from_le_bytes(self.get(offset..offset + size_of::<$t>()).ok_or_else(#[cold] || format!("error reading field"))?.try_into().unwrap()))
		}
		fn [<read_ $t _be>](&self, offset: usize) -> Result<$t, String> {
			Ok($t::from_be_bytes(self.get(offset..offset + size_of::<$t>()).ok_or_else(#[cold] || format!("error reading field"))?.try_into().unwrap()))
		}
		fn [<get_ $t _at>](&self, offset: usize) -> Option<$t> {
			Some($t::from_le_bytes(self.get(offset..offset + size_of::<$t>())?.try_into().unwrap()))
		}
		fn [<get_ $t _at_be>](&self, offset: usize) -> Option<$t> {
			Some($t::from_be_bytes(self.get(offset..offset + size_of::<$t>())?.try_into().unwrap()))
		}
	)*}}
}

pub trait ByteSlice {
	fn starts_with_at(&self, needle: &[u8], offset: usize) -> bool;
	fn read_bytes(&self, offset: usize, len: usize, name: &str) -> Result<&[u8], String>;
	fn read_u8(&self, offset: usize) -> Result<u8, String>;
	fn read_u16(&self, offset: usize) -> Result<u16, String>;
	fn read_u32(&self, offset: usize) -> Result<u32, String>;
	fn read_u64(&self, offset: usize) -> Result<u64, String>;
	fn read_usize(&self, offset: usize) -> Result<usize, String>;
	fn read_i8(&self, offset: usize) -> Result<i8, String>;
	fn read_i16(&self, offset: usize) -> Result<i16, String>;
	fn read_i32(&self, offset: usize) -> Result<i32, String>;
	fn read_i64(&self, offset: usize) -> Result<i64, String>;
	fn read_isize(&self, offset: usize) -> Result<isize, String>;
	fn read_u8_be(&self, offset: usize) -> Result<u8, String>;
	fn read_u16_be(&self, offset: usize) -> Result<u16, String>;
	fn read_u32_be(&self, offset: usize) -> Result<u32, String>;
	fn read_u64_be(&self, offset: usize) -> Result<u64, String>;
	fn read_usize_be(&self, offset: usize) -> Result<usize, String>;
	fn read_i8_be(&self, offset: usize) -> Result<i8, String>;
	fn read_i16_be(&self, offset: usize) -> Result<i16, String>;
	fn read_i32_be(&self, offset: usize) -> Result<i32, String>;
	fn read_i64_be(&self, offset: usize) -> Result<i64, String>;
	fn read_isize_be(&self, offset: usize) -> Result<isize, String>;
	fn get_u8_at(&self, offset: usize) -> Option<u8>;
	fn get_u16_at(&self, offset: usize) -> Option<u16>;
	fn get_u32_at(&self, offset: usize) -> Option<u32>;
	fn get_u64_at(&self, offset: usize) -> Option<u64>;
	fn get_usize_at(&self, offset: usize) -> Option<usize>;
	fn get_i8_at(&self, offset: usize) -> Option<i8>;
	fn get_i16_at(&self, offset: usize) -> Option<i16>;
	fn get_i32_at(&self, offset: usize) -> Option<i32>;
	fn get_i64_at(&self, offset: usize) -> Option<i64>;
	fn get_isize_at(&self, offset: usize) -> Option<isize>;
	fn get_u8_at_be(&self, offset: usize) -> Option<u8>;
	fn get_u16_at_be(&self, offset: usize) -> Option<u16>;
	fn get_u32_at_be(&self, offset: usize) -> Option<u32>;
	fn get_u64_at_be(&self, offset: usize) -> Option<u64>;
	fn get_usize_at_be(&self, offset: usize) -> Option<usize>;
	fn get_i8_at_be(&self, offset: usize) -> Option<i8>;
	fn get_i16_at_be(&self, offset: usize) -> Option<i16>;
	fn get_i32_at_be(&self, offset: usize) -> Option<i32>;
	fn get_i64_at_be(&self, offset: usize) -> Option<i64>;
	fn get_isize_at_be(&self, offset: usize) -> Option<isize>;
	fn unswizzled_psp(&self, width: u32, height: u32) -> Vec<u8>;
}

impl ByteSlice for [u8] {
	fn starts_with_at(&self, needle: &[u8], offset: usize) -> bool {
		self.get(offset..offset + needle.len()).map_or(false, |x| x.starts_with(needle))
	}

	fn read_bytes(&self, offset: usize, len: usize, name: &str) -> Result<&[u8], String> {
		Ok(self.get(offset..offset + len).ok_or_else(#[cold] || format!("error reading {name}"))?.try_into().unwrap())
	}

	impl_byte_readers!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);

	fn unswizzled_psp(&self, width_bytes: u32, height: u32) -> Vec<u8> {
		let mut pixels = Vec::with_capacity(self.len());
		let blocks_per_row = width_bytes / 16;
		for y in 0..height {
			for x in 0..width_bytes {
				let block_idx_x = x / 16;
				let block_idx_y = y / 8;
				let x_in_block = x % 16;
				let y_in_block = y % 8;
				let block_start = (block_idx_y * blocks_per_row + block_idx_x) * (16 * 8);
				let swizzled_idx = block_start + y_in_block * 16 + x_in_block;
				pixels.push(self.get(swizzled_idx as usize).cloned().unwrap_or_default());
			}
		}
		pixels
	}
}