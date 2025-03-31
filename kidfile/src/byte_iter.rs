use std::mem::MaybeUninit;

pub trait ByteIter: Iterator<Item = u8> + ExactSizeIterator {
	fn next_bytes<const LEN: usize>(&mut self) -> Option<[u8; LEN]>;
	fn next_u16(&mut self) -> Option<u16>;
	fn next_u32(&mut self) -> Option<u32>;
	fn next_u64(&mut self) -> Option<u64>;
	fn next_usize(&mut self) -> Option<usize>;
	fn next_i8(&mut self) -> Option<i8>;
	fn next_i16(&mut self) -> Option<i16>;
	fn next_i32(&mut self) -> Option<i32>;
	fn next_i64(&mut self) -> Option<i64>;
	fn next_isize(&mut self) -> Option<isize>;
	fn next_u16_be(&mut self) -> Option<u16>;
	fn next_u32_be(&mut self) -> Option<u32>;
	fn next_u64_be(&mut self) -> Option<u64>;
	fn next_usize_be(&mut self) -> Option<usize>;
	fn next_i8_be(&mut self) -> Option<i8>;
	fn next_i16_be(&mut self) -> Option<i16>;
	fn next_i32_be(&mut self) -> Option<i32>;
	fn next_i64_be(&mut self) -> Option<i64>;
	fn next_isize_be(&mut self) -> Option<isize>;
}

macro_rules! impl_for_types {
	($($t:ty),*) => {paste::paste! {$(
		fn [<next_ $t>](&mut self) -> Option<$t> {
			self.next_bytes::<{size_of::<$t>()}>().map(|x| $t::from_le_bytes(x))
		}
		fn [<next_ $t _be>](&mut self) -> Option<$t> {
			self.next_bytes::<{size_of::<$t>()}>().map(|x| $t::from_be_bytes(x))
		}
	)*}}
}

impl<T: Iterator<Item = u8> + ExactSizeIterator> ByteIter for T {
	fn next_bytes<const LEN: usize>(&mut self) -> Option<[u8; LEN]> {
		if LEN <= self.len() {
			let mut arr = unsafe {MaybeUninit::<[u8; LEN]>::uninit().assume_init()};
			for (i, x) in self.take(LEN).enumerate() {
				arr[i] = x;
			}
			Some(arr)
		} else {
			None
		}
	}

	impl_for_types!(u16, u32, u64, usize, i8, i16, i32, i64, isize);
}