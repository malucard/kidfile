use std::{borrow::Cow, collections::VecDeque, fs::{self}, sync::{atomic::{self, AtomicBool, AtomicUsize}, Arc, RwLock}, thread::{self, JoinHandle}};
use egui::{Align, Button, Context, Id, Label, Layout, Modal, ProgressBar, TextEdit};
use image::ExtendedColorType;
use kidfile::{auto_decode_step, DynData};
use crate::{complex_path::ComplexPath, dirty_config, BATCH_CONVERT_IMAGES, BATCH_DECOMPRESS, BATCH_EXTRACT_ARCHIVES, EXTRACTION_SUFFIX};

enum BatchStatus {
	Configuring,
	Running,
	Finished
}

pub struct BatchDecode {
	status: BatchStatus,
	path: ComplexPath,
	extraction_suffix: String,
	extract_archives: bool,
	decompress: bool,
	convert_images: bool,
	pending_files: Arc<RwLock<VecDeque<ComplexPath>>>,
	found_file_count: Arc<AtomicUsize>,
	processed_file_count: Arc<AtomicUsize>,
	threads: Vec<JoinHandle<()>>,
	cancel: Arc<AtomicBool>
}

fn survey(files: &mut VecDeque<ComplexPath>, path: &ComplexPath) {
	path.iterate(|name, is_dir| {
		if is_dir {
			survey(files, &mut path.join_dir(&name));
		} else {
			files.push_back(path.join_file(name));
		}
	});
}

impl BatchDecode {
	pub fn new(path: ComplexPath) -> Self {
		Self {
			status: BatchStatus::Configuring,
			path,
			extraction_suffix: EXTRACTION_SUFFIX.read().unwrap().clone(),
			extract_archives: BATCH_EXTRACT_ARCHIVES.load(atomic::Ordering::Acquire),
			decompress: BATCH_DECOMPRESS.load(atomic::Ordering::Acquire),
			convert_images: BATCH_CONVERT_IMAGES.load(atomic::Ordering::Acquire),
			pending_files: Arc::new(RwLock::new(VecDeque::new())),
			found_file_count: Arc::new(AtomicUsize::new(0)),
			processed_file_count: Arc::new(AtomicUsize::new(0)),
			threads: Vec::new(),
			cancel: Arc::new(AtomicBool::new(false))
		}
	}

	fn is_suffix_valid(&self) -> bool {
		self.extraction_suffix.len() > 0
			&& !self.extraction_suffix.contains(|c| matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || c.is_control())
			&& !matches!(self.extraction_suffix.chars().last().unwrap(), ' ' | '.')
	}

	fn start_decoding(&mut self) {
		self.found_file_count.store(self.pending_files.read().unwrap().len(), atomic::Ordering::SeqCst);
		self.processed_file_count.store(0, atomic::Ordering::SeqCst);
		let root_parent = Arc::new(self.path.get_physical().parent().unwrap().to_path_buf());
		for _ in 0..num_cpus::get() {
			let root_parent = root_parent.clone();
			let extraction_suffix = self.extraction_suffix.clone();
			let extract_archives = self.extract_archives;
			let decompress = self.decompress;
			let convert_images = self.convert_images;
			let pending_files = self.pending_files.clone();
			let found_file_count = self.found_file_count.clone();
			let processed_file_count = self.processed_file_count.clone();
			let cancel = self.cancel.clone();
			self.threads.push(thread::spawn(move || {
				while let Some(path) = {pending_files.write().unwrap().pop_front()} {
					let mut target = root_parent.to_path_buf();
					let tmp_path = path.to_physical();
					let components = tmp_path.strip_prefix(&*root_parent).unwrap().components().collect::<Vec<_>>();
					for (i, component) in components.iter().enumerate() {
						if i == components.len() - 1 {
							target.push(component.as_os_str());
						} else {
							let mut component = component.as_os_str().to_owned();
							component.push(&extraction_suffix);
							target.push(component);
						}
					}
					if let Ok(mut data) = path.load() {
						let mut steps_taken = Vec::new();
						loop {
							if cancel.load(atomic::Ordering::Acquire) {
								return;
							}
							match auto_decode_step(data.to_mut(), path.get_archive_format(), path.get_archive_format()) {
								Ok((step, DynData::Raw(raw))) => {
									steps_taken.push(step);
									data = Cow::Owned(raw);
								}
								Ok((_, DynData::Image(img))) => {
									fs::create_dir_all(&target.parent().unwrap()).unwrap();
									if convert_images {
										for (i, frame) in img.frames.iter().enumerate() {
											let mut file_name = target.file_name().unwrap().to_owned();
											if i > 0 {
												file_name.push(format!(".{i}.png"));
											} else {
												file_name.push(".png");
											}
											image::save_buffer(target.with_file_name(file_name), frame.as_rgba_bytes(), frame.width, frame.height, ExtendedColorType::Rgba8).unwrap();
										}
									} else if decompress && (!steps_taken.is_empty() || !path.is_physical()) {
										fs::write(&target, data.to_mut().read()).unwrap();
									}
									break;
								}
								Ok((_, DynData::Archive(arc))) => {
									if extract_archives {
										let arc = Arc::new(arc);
										let arc_path = path.clone().into_archive(arc);
										let mut pending_files = pending_files.write().unwrap();
										arc_path.iterate(|name, _| {
											pending_files.push_back(arc_path.join_file(name));
											found_file_count.fetch_add(1, atomic::Ordering::SeqCst);
										});
									} else if decompress && (!steps_taken.is_empty() || !path.is_physical()) {
										fs::create_dir_all(&target.parent().unwrap()).unwrap();
										fs::write(&target, data.to_mut().read()).unwrap();
									}
									break;
								}
								Err(_) => {
									if decompress && (!steps_taken.is_empty() || !path.is_physical()) {
										fs::create_dir_all(&target.parent().unwrap()).unwrap();
										fs::write(&target, data.to_mut().read()).unwrap();
									}
									break;
								}
							}
						}
					}
					processed_file_count.fetch_add(1, atomic::Ordering::SeqCst);
				}
			}));
		}
	}

	fn clean_threads(&mut self) {
		self.threads.retain(|thread| !thread.is_finished());
	}

	pub fn show(&mut self, ctx: &Context) -> bool {
		let mut allow_closing = true;
		let mut close = false;
		let wants_to_close = match self.status {
			BatchStatus::Configuring => Modal::new(Id::new("batch_config_modal")).show(ctx, |ui| {
				ui.set_width(480.0);
				ui.vertical_centered_justified(|ui| {
					ui.label("Batch Decode...");
					ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
						let mut path_str = self.path.file_name().map_or_else(|| self.path.to_str().into_owned(), |x| x.to_string_lossy().into_owned());
						if let Some(parent) = self.path.parent() {
							let parent_str = parent.file_name().map_or_else(|| parent.to_str().into_owned(), |x| x.to_string_lossy().into_owned());
							path_str = format!("{}/{}", parent_str, path_str);
						}
						ui.add(Label::new(format!("Source: .../{}", path_str)).wrap());
						ui.add(Label::new(format!("Target: .../{}{}", path_str, self.extraction_suffix)).wrap());
						ui.separator();
						ui.horizontal(|ui| {
							ui.label("Extraction suffix");
							let suffix_valid = self.is_suffix_valid();
							let mut edit = TextEdit::singleline(&mut self.extraction_suffix).char_limit(64).code_editor();
							if !suffix_valid {
								edit = edit.background_color(ui.visuals().error_fg_color.gamma_multiply(0.25));
							}
							ui.add(edit);
						});
						ui.checkbox(&mut self.extract_archives, "Unpack archives");
						ui.checkbox(&mut self.decompress, "Decompress compressed files");
						ui.checkbox(&mut self.convert_images, "Convert images");
					});
					ui.separator();
					ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
						if ui.button("Cancel").clicked() {
							close = true;
						}
						if ui.button("Start").clicked() && self.is_suffix_valid() && self.path.can_iterate() {
							let mut files = VecDeque::new();
							survey(&mut files, &self.path);
							*self.pending_files.write().unwrap() = files;
							self.start_decoding();
							self.status = BatchStatus::Running;
							*EXTRACTION_SUFFIX.write().unwrap() = self.extraction_suffix.clone();
							BATCH_EXTRACT_ARCHIVES.store(self.extract_archives, atomic::Ordering::Release);
							BATCH_DECOMPRESS.store(self.decompress, atomic::Ordering::Release);
							BATCH_CONVERT_IMAGES.store(self.convert_images, atomic::Ordering::Release);
							dirty_config();
							allow_closing = false;
							ui.ctx().request_repaint();
						}
					});
				});
			}).should_close(),
			BatchStatus::Running | BatchStatus::Finished => Modal::new(Id::new("batch_running_modal")).show(ctx, |ui| {
				allow_closing = false;
				ui.set_width(480.0);
				ui.vertical_centered_justified(|ui| {
					ui.label("Running...");
					ui.separator();
					let found = self.found_file_count.load(atomic::Ordering::Acquire);
					let processed = self.processed_file_count.load(atomic::Ordering::Acquire);
					ui.label(format!("{} threads running", self.threads.len()));
					ui.label(format!("Found {} files", found));
					ui.label(format!("Processed {} files", processed));
					ui.add(ProgressBar::new(processed as f32 / found as f32));
					ui.separator();
					let already_canceling = self.cancel.load(atomic::Ordering::Acquire);
					if let BatchStatus::Finished = self.status {
						ui.label("Done");
						if ui.add(Button::new("OK").small()).clicked() {
							close = true;
						}
					} else {
						if ui.add_enabled(!already_canceling, Button::new("Cancel").small()).clicked() {
							self.cancel.store(true, atomic::Ordering::Release);
							close = true;
						}
					}
				});
				self.clean_threads();
				if self.threads.is_empty() {
					allow_closing = true;
					self.status = BatchStatus::Finished;
				}
				ui.ctx().request_repaint();
			}).should_close()
		};
		allow_closing && (wants_to_close || close)
	}
}