use std::{borrow::Cow, ffi::{OsStr, OsString}, path::{Path, PathBuf, StripPrefixError}, sync::Arc};
use kidfile::{file_data::FileData, Archive};

#[derive(Clone)]
pub enum ComplexPath {
	Physical(PathBuf, bool),
	Archive(PathBuf, Vec<(Arc<Archive>, String)>, Arc<Archive>, String)
}

impl From<PathBuf> for ComplexPath {
	fn from(value: PathBuf) -> Self {
		let is_dir = value.is_dir();
		Self::Physical(value, is_dir)
	}
}

impl ComplexPath {
	pub fn to_str(&self) -> Cow<str> {
		match self {
			Self::Physical(p, _) => p.to_string_lossy(),
			Self::Archive(physical, inner, _, subfile) => {
				if subfile.is_empty() {
					if inner.is_empty() {
						physical.to_string_lossy()
					} else {
						Cow::Owned(format!("{}:{}", physical.to_string_lossy(), inner.iter().fold(String::new(), |x, y| format!("{x}:{}", y.1))))
					}
				} else {
					Cow::Owned(format!("{}:{}:{}", physical.to_string_lossy(), inner.iter().fold(String::new(), |x, y| format!("{x}:{}", y.1)), subfile))
				}
			}
		}
	}

	pub fn is_dir_or_archive(&self) -> bool {
		match self {
			Self::Physical(_, is_dir) => *is_dir,
			Self::Archive(_, _, _, subfile) => subfile.is_empty()
		}
	}

	pub fn can_iterate(&self) -> bool {
		match self {
			Self::Physical(p, is_dir) => *is_dir && p.read_dir().is_ok(),
			Self::Archive(..) => true
		}
	}

	pub fn get_physical(&self) -> &Path {
		match self {
			Self::Physical(p, _) => p,
			Self::Archive(physical, _, _, _) => physical
		}
	}

	pub fn iterate<F: FnMut(OsString, bool)>(&self, mut f: F) {
		match self {
			Self::Physical(p, is_dir) => {
				assert!(is_dir);
				for entry in p.read_dir().unwrap() {
					if let Ok(entry) = entry {
						if let Ok(file_type) = entry.file_type() {
							f(entry.file_name(), file_type.is_dir());
						}
					}
				}
			}
			Self::Archive(_, _, arc, subfile) => {
				assert!(subfile.is_empty());
				for entry in arc.entries.iter() {
					f(OsString::from(&entry.name), false);
				}
			}
		}
	}

	pub fn is_physical(&self) -> bool {
		matches!(self, Self::Physical(..))
	}

	pub fn get_archive_format(&self) -> Option<&'static str> {
		match self {
			Self::Archive(_, _, arc, _) => Some(arc.format),
			_ => None
		}
	}

	pub fn append_dir(&mut self, sub: &OsStr) {
		match self {
			Self::Physical(p, is_dir) => {
				assert!(*is_dir);
				*p = p.join(sub);
			}
			_ => unreachable!()
		}
	}

	pub fn append_archive(&mut self, sub: &OsStr, arc: Archive) {
		match self {
			Self::Physical(p, is_dir) => {
				assert!(*is_dir);
				*self = Self::Archive(p.join(sub), Vec::new(), Arc::new(arc), String::new());
			}
			Self::Archive(_, inner, prev_arc, subfile) => {
				assert!(subfile.is_empty());
				inner.push((std::mem::replace(prev_arc, Arc::new(arc)), sub.to_string_lossy().into()));
			}
		}
	}

	pub fn into_archive(self, arc: Arc<Archive>) -> Self {
		match self {
			Self::Physical(p, is_dir) => {
				assert!(!is_dir);
				Self::Archive(p, Vec::new(), arc, String::new())
			}
			Self::Archive(physical, mut inner, prev_arc, subfile) => {
				assert!(!subfile.is_empty());
				inner.push((prev_arc, subfile));
				Self::Archive(physical, inner, arc, String::new())
			}
		}
	}

	pub fn join_dir(&self, sub: &OsStr) -> Self {
		match self {
			Self::Physical(p, is_dir) => {
				assert!(*is_dir);
				Self::Physical(p.join(sub), true)
			}
			_ => panic!()
		}
	}

	pub fn join_file(&self, file_name: OsString) -> Self {
		match self {
			Self::Physical(p, is_dir) => {
				assert!(*is_dir);
				Self::Physical(p.join(file_name), false)
			}
			Self::Archive(physical, inner, arc, subfile) => {
				assert!(subfile.is_empty());
				Self::Archive(physical.clone(), inner.clone(), arc.clone(), file_name.to_string_lossy().into_owned())
			}
		}
	}

	pub fn to_str_stripping_dir_prefix(&self, prefix: &Path) -> Result<Cow<str>, StripPrefixError> {
		match self {
			Self::Physical(p, _) => Ok(p.strip_prefix(prefix)?.to_string_lossy()),
			Self::Archive(physical, inner, _, subfile) => {
				if subfile.is_empty() {
					if inner.is_empty() {
						Ok(physical.strip_prefix(prefix)?.to_string_lossy())
					} else {
						Ok(Cow::Owned(format!("{}:{}", physical.strip_prefix(prefix)?.to_string_lossy(), inner.iter().fold(String::new(), |x, y| format!("{x}:{}", y.1)))))
					}
				} else {
					Ok(Cow::Owned(format!("{}:{}:{}", physical.strip_prefix(prefix)?.to_string_lossy(), inner.iter().fold(String::new(), |x, y| format!("{x}:{}", y.1)), subfile)))
				}
			}
		}
	}

	//pub fn join_archive(&self, sub: OsString) -> Self {
	//	todo!()
	//}

	pub fn file_name(&self) -> Option<Cow<OsStr>> {
		match self {
			Self::Physical(p, _) => p.file_name().map(|x| Cow::Borrowed(x)),
			Self::Archive(physical, inner, _, subfile) => {
				if subfile.is_empty() {
					if inner.is_empty() {
						physical.file_name().map(|x| Cow::Borrowed(x))
					} else {
						inner.last().map(|x| Cow::Owned(OsString::from(&x.1)))
					}
				} else {
					Some(Cow::Owned(subfile.as_str().into()))
				}
			}
		}
	}

	pub fn to_physical(&self) -> Cow<PathBuf> {
		match self {
			Self::Physical(p, _) => Cow::Borrowed(p),
			Self::Archive(physical, inner, _, subfile) => {
				let mut out = physical.clone();
				for (_, inner) in inner {
					out.push(inner);
				}
				if !subfile.is_empty() {
					out.push(subfile);
				}
				Cow::Owned(out)
			}
		}
	}

	pub fn parent(&self) -> Option<Self> {
		match self {
			Self::Physical(p, _) => p.parent().map(|x| Self::Physical(x.into(), true)),
			Self::Archive(physical, inner, arc, subfile) => {
				if subfile.is_empty() {
					if let Some(((parent_arc, _), rest)) = inner.split_last() {
						Some(Self::Archive(physical.clone(), rest.into(), parent_arc.clone(), String::new()))
					} else {
						physical.parent().map(|x| Self::Physical(x.into(), true))
					}
				} else {
					Some(Self::Archive(physical.clone(), inner.clone(), arc.clone(), String::new()))
				}
			}
		}
	}

	pub fn load_file(&self, name: &OsStr) -> Result<Cow<FileData>, ()> {
		match self {
			Self::Physical(p, is_dir) => {
				assert!(*is_dir);
				let path = p.join(name);
				let size = std::fs::metadata(&path).map_err(|_| ())?.len() as usize;
				Ok(Cow::Owned(FileData::Stream {path, file: None, start: 0, size}))
			}
			Self::Archive(_, _, arc, subfile) => {
				assert!(subfile.is_empty());
				let name = name.to_string_lossy();
				for e in arc.entries.iter() {
					if e.name == name {
						return Ok(Cow::Borrowed(&e.data));
					}
				}
				Err(())
			}
		}
	}

	pub fn load(&self) -> Result<Cow<FileData>, ()> {
		match self {
			Self::Physical(p, is_dir) => {
				assert!(!*is_dir);
				let size = std::fs::metadata(&p).map_err(|_| ())?.len() as usize;
				Ok(Cow::Owned(FileData::Stream {path: p.clone(), file: None, start: 0, size}))
			}
			Self::Archive(_, _, arc, subfile) => {
				assert!(!subfile.is_empty());
				for e in arc.entries.iter() {
					if e.name == *subfile {
						return Ok(Cow::Borrowed(&e.data));
					}
				}
				Err(())
			}
		}
	}
}