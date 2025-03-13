pub trait ByteSlice {
	fn get_u8_at(&self, offset: usize) -> Option<u8>;
	fn get_u16_at(&self, offset: usize) -> Option<u16>;
	fn get_u32_at(&self, offset: usize) -> Option<u32>;
	fn get_u64_at(&self, offset: usize) -> Option<u64>;
	fn get_i8_at(&self, offset: usize) -> Option<i8>;
	fn get_i16_at(&self, offset: usize) -> Option<i16>;
	fn get_i32_at(&self, offset: usize) -> Option<i32>;
	fn get_i64_at(&self, offset: usize) -> Option<i64>;
	fn get_u8_be_at(&self, offset: usize) -> Option<u8>;
	fn get_u16_be_at(&self, offset: usize) -> Option<u16>;
	fn get_u32_be_at(&self, offset: usize) -> Option<u32>;
	fn get_u64_be_at(&self, offset: usize) -> Option<u64>;
	fn get_i8_be_at(&self, offset: usize) -> Option<i8>;
	fn get_i16_be_at(&self, offset: usize) -> Option<i16>;
	fn get_i32_be_at(&self, offset: usize) -> Option<i32>;
	fn get_i64_be_at(&self, offset: usize) -> Option<i64>;
}

impl ByteSlice for &[u8] {
	fn get_u8_at(&self, offset: usize) -> Option<u8> {
		Some(u8::from_le_bytes(self.get(offset..offset + 1)?.try_into().ok()?))
	}
	fn get_u16_at(&self, offset: usize) -> Option<u16> {
		Some(u16::from_le_bytes(self.get(offset..offset + 2)?.try_into().ok()?))
	}
	fn get_u32_at(&self, offset: usize) -> Option<u32> {
		Some(u32::from_le_bytes(self.get(offset..offset + 4)?.try_into().ok()?))
	}
	fn get_u64_at(&self, offset: usize) -> Option<u64> {
		Some(u64::from_le_bytes(self.get(offset..offset + 8)?.try_into().ok()?))
	}
	fn get_i8_at(&self, offset: usize) -> Option<i8> {
		Some(i8::from_le_bytes(self.get(offset..offset + 1)?.try_into().ok()?))
	}
	fn get_i16_at(&self, offset: usize) -> Option<i16> {
		Some(i16::from_le_bytes(self.get(offset..offset + 2)?.try_into().ok()?))
	}
	fn get_i32_at(&self, offset: usize) -> Option<i32> {
		Some(i32::from_le_bytes(self.get(offset..offset + 4)?.try_into().ok()?))
	}
	fn get_i64_at(&self, offset: usize) -> Option<i64> {
		Some(i64::from_le_bytes(self.get(offset..offset + 8)?.try_into().ok()?))
	}
	fn get_u8_be_at(&self, offset: usize) -> Option<u8> {
		Some(u8::from_be_bytes(self.get(offset..offset + 1)?.try_into().ok()?))
	}
	fn get_u16_be_at(&self, offset: usize) -> Option<u16> {
		Some(u16::from_be_bytes(self.get(offset..offset + 2)?.try_into().ok()?))
	}
	fn get_u32_be_at(&self, offset: usize) -> Option<u32> {
		Some(u32::from_be_bytes(self.get(offset..offset + 4)?.try_into().ok()?))
	}
	fn get_u64_be_at(&self, offset: usize) -> Option<u64> {
		Some(u64::from_be_bytes(self.get(offset..offset + 8)?.try_into().ok()?))
	}
	fn get_i8_be_at(&self, offset: usize) -> Option<i8> {
		Some(i8::from_be_bytes(self.get(offset..offset + 1)?.try_into().ok()?))
	}
	fn get_i16_be_at(&self, offset: usize) -> Option<i16> {
		Some(i16::from_be_bytes(self.get(offset..offset + 2)?.try_into().ok()?))
	}
	fn get_i32_be_at(&self, offset: usize) -> Option<i32> {
		Some(i32::from_be_bytes(self.get(offset..offset + 4)?.try_into().ok()?))
	}
	fn get_i64_be_at(&self, offset: usize) -> Option<i64> {
		Some(i64::from_be_bytes(self.get(offset..offset + 8)?.try_into().ok()?))
	}
}