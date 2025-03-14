macro_rules! impl_byte_readers {
	($($t:ty),*) => {paste::paste! {$(
		fn [<read_ $t _field>](&self, offset: usize, name: &'static str) -> Result<$t, String> {
			Ok($t::from_le_bytes(self.get(offset..offset + size_of::<$t>()).ok_or_else(#[cold] || format!("could not read {name}"))?.try_into().unwrap()))
		}
		fn [<read_ $t _field_be>](&self, offset: usize, name: &'static str) -> Result<$t, String> {
			Ok($t::from_be_bytes(self.get(offset..offset + size_of::<$t>()).ok_or_else(#[cold] || format!("could not read {name}"))?.try_into().unwrap()))
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
	fn read_u8_field(&self, offset: usize, name: &'static str) -> Result<u8, String>;
	fn read_u16_field(&self, offset: usize, name: &'static str) -> Result<u16, String>;
	fn read_u32_field(&self, offset: usize, name: &'static str) -> Result<u32, String>;
	fn read_u64_field(&self, offset: usize, name: &'static str) -> Result<u64, String>;
	fn read_usize_field(&self, offset: usize, name: &'static str) -> Result<usize, String>;
	fn read_i8_field(&self, offset: usize, name: &'static str) -> Result<i8, String>;
	fn read_i16_field(&self, offset: usize, name: &'static str) -> Result<i16, String>;
	fn read_i32_field(&self, offset: usize, name: &'static str) -> Result<i32, String>;
	fn read_i64_field(&self, offset: usize, name: &'static str) -> Result<i64, String>;
	fn read_isize_field(&self, offset: usize, name: &'static str) -> Result<isize, String>;
	fn read_u8_field_be(&self, offset: usize, name: &'static str) -> Result<u8, String>;
	fn read_u16_field_be(&self, offset: usize, name: &'static str) -> Result<u16, String>;
	fn read_u32_field_be(&self, offset: usize, name: &'static str) -> Result<u32, String>;
	fn read_u64_field_be(&self, offset: usize, name: &'static str) -> Result<u64, String>;
	fn read_usize_field_be(&self, offset: usize, name: &'static str) -> Result<usize, String>;
	fn read_i8_field_be(&self, offset: usize, name: &'static str) -> Result<i8, String>;
	fn read_i16_field_be(&self, offset: usize, name: &'static str) -> Result<i16, String>;
	fn read_i32_field_be(&self, offset: usize, name: &'static str) -> Result<i32, String>;
	fn read_i64_field_be(&self, offset: usize, name: &'static str) -> Result<i64, String>;
	fn read_isize_field_be(&self, offset: usize, name: &'static str) -> Result<isize, String>;
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
}

impl ByteSlice for &[u8] {
	impl_byte_readers!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
}