use std::{borrow::Cow, cmp::Ordering, collections::HashMap, ffi::OsString, fs::File, io::{BufReader, Write}, path::{Path, PathBuf}, sync::{atomic::{self, AtomicBool, AtomicUsize}, LazyLock, RwLock}, time::{Duration, Instant}};
use batch_decode::BatchDecode;
use complex_path::ComplexPath;
use data_view::DataView;
use egui::{epaint::text::{FontInsert, FontPriority, InsertFontFamily}, popup, vec2, Align, Button, CentralPanel, Context, FontData, FontFamily, Grid, Key, Label, Layout, Modifiers, PopupCloseBehavior, Pos2, Rect, ScrollArea, Separator, TextStyle, TextWrapMode, TextureOptions, TopBottomPanel, Ui, UiBuilder, Vec2, ViewportBuilder};
use egui_dock::{DockArea, DockState, NodeIndex, SurfaceIndex, TabAddAlign, TabViewer};
use kidfile::{auto_decode_full, file_data::FileData, DynData};
use rfd::FileDialog;
use serde_json::Value;

mod batch_decode;
mod complex_path;
mod data_view;

pub static LOGS: RwLock<Vec<String>> = RwLock::new(vec![]);

#[macro_export]
macro_rules! log {
	($($e:expr),+) => {{
		let s = format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), format!($($e),+)).trim().into();
		println!("{}", s);
		crate::LOGS.write().unwrap().push(s);
	}};
}

fn get_last_log() -> String {
	LOGS.read().unwrap().last().cloned().unwrap_or_else(|| "".into())
}

static CONFIG_FILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
	let dirs = directories::ProjectDirs::from("", "", env!("CARGO_PKG_NAME")).unwrap();
	let dir = dirs.config_local_dir();
	std::fs::create_dir_all(dir).unwrap();
	dir.join("config.json")
});

#[allow(deprecated)]
static HOME_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(std::env::home_dir);
static LAST_ACCESSED_PATH: RwLock<Option<PathBuf>> = RwLock::new(None);
static BOOKMARKS: RwLock<Vec<PathBuf>> = RwLock::new(Vec::new());
static EXTRACTION_SUFFIX: LazyLock<RwLock<String>> = LazyLock::new(|| RwLock::new("~".into()));
static BATCH_EXTRACT_ARCHIVES: AtomicBool = AtomicBool::new(true);
static BATCH_DECOMPRESS: AtomicBool = AtomicBool::new(true);
static BATCH_CONVERT_IMAGES: AtomicBool = AtomicBool::new(true);

static IS_CONFIG_DIRTY: AtomicBool = AtomicBool::new(false);
static LAST_SAVE_TIME: LazyLock<RwLock<Instant>> = LazyLock::new(|| RwLock::new(Instant::now()));

fn read_config() {
	*LAST_ACCESSED_PATH.write().unwrap() = HOME_DIR.clone();
	if let Ok(config_file) = File::open(CONFIG_FILE_PATH.as_path()) {
		let buf: HashMap<String, Value> = serde_json::from_reader(BufReader::new(config_file)).unwrap();

		if let Some(Value::String(value)) = buf.get("last_accessed_path") {
			let path = PathBuf::from(value);
			if path.is_dir() && path.read_dir().is_ok() {
				*LAST_ACCESSED_PATH.write().unwrap() = Some(path);
			}
		}

		if let Some(Value::Array(value)) = buf.get("bookmarks") {
			let mut bookmarks = BOOKMARKS.write().unwrap();
			for entry in value {
				if let Value::String(path) = entry {
					bookmarks.push(path.into());
				}
			}
		}

		if let Some(Value::String(value)) = buf.get("extraction_suffix") {
			*EXTRACTION_SUFFIX.write().unwrap() = value.clone();
		}

		if let Some(Value::Bool(value)) = buf.get("batch_extract") {
			BATCH_EXTRACT_ARCHIVES.store(*value, atomic::Ordering::Release);
		}

		if let Some(Value::Bool(value)) = buf.get("batch_decompress") {
			BATCH_DECOMPRESS.store(*value, atomic::Ordering::Release);
		}

		if let Some(Value::Bool(value)) = buf.get("batch_convert_images") {
			BATCH_CONVERT_IMAGES.store(*value, atomic::Ordering::Release);
		}
	}
}

fn write_config() {
	let now = Instant::now();
	if IS_CONFIG_DIRTY.load(atomic::Ordering::Acquire) && now - *LAST_SAVE_TIME.read().unwrap() > Duration::from_secs(1) {
		IS_CONFIG_DIRTY.store(false, atomic::Ordering::Release);
		*LAST_SAVE_TIME.write().unwrap() = now;
		let mut config = HashMap::new();

		let last_accessed_path = LAST_ACCESSED_PATH.read().unwrap();
		let last_accessed_path = last_accessed_path.as_ref().map_or(Path::new(""), |x| x.as_path());
		config.insert("last_accessed_path", serde_json::to_value(last_accessed_path).unwrap());

		let bookmarks = BOOKMARKS.read().unwrap();
		config.insert("bookmarks", serde_json::to_value(&*bookmarks).unwrap());

		config.insert("extraction_suffix", serde_json::to_value(&*EXTRACTION_SUFFIX.read().unwrap()).unwrap());

		config.insert("batch_extract", serde_json::to_value(BATCH_EXTRACT_ARCHIVES.load(atomic::Ordering::Acquire)).unwrap());

		config.insert("batch_decompress", serde_json::to_value(BATCH_DECOMPRESS.load(atomic::Ordering::Acquire)).unwrap());

		config.insert("batch_convert_images", serde_json::to_value(BATCH_CONVERT_IMAGES.load(atomic::Ordering::Acquire)).unwrap());

		let buf = serde_json::to_string_pretty(&config).unwrap();
		File::create(CONFIG_FILE_PATH.as_path()).unwrap().write_all(buf.as_bytes()).unwrap();
	}
}

fn dirty_config() {
	IS_CONFIG_DIRTY.store(true, atomic::Ordering::Release);
}

static UNIQUE_ID: AtomicUsize = AtomicUsize::new(0);

struct ExplorerEntry {
	pub name: OsString,
	pub is_dir: bool
}

struct ExplorerTab {
	pub id: usize,
	pub path: ComplexPath,
	pub children: Vec<ExplorerEntry>,
	pub selection: Option<(usize, DynData, Vec<&'static str>)>,
	pub view: DataView,
	pub selection_size: Option<usize>,
	pub past: Vec<ComplexPath>,
	pub future: Vec<ComplexPath>,
	pub batch_task: Option<BatchDecode>
}

impl ExplorerTab {
	pub fn new(ctx: &Context, folder: PathBuf) -> Self {
		let mut tab = ExplorerTab {
			id: UNIQUE_ID.fetch_add(1, atomic::Ordering::SeqCst),
			children: Vec::new(),
			path: ComplexPath::Physical(folder, true),
			selection: None,
			selection_size: None,
			view: DataView::None,
			past: Vec::new(),
			future: Vec::new(),
			batch_task: None
		};
		tab.refresh(ctx);
		tab
	}

	pub fn refresh(&mut self, ctx: &Context) {
		let selected_name = self.selection.as_ref().map(|s| self.children[s.0].name.clone());
		self.children.clear();
		self.path.iterate(|name, is_dir| {
			self.children.push(ExplorerEntry {name, is_dir});
		});
		self.children.sort_by(|x, y|
			if x.is_dir != y.is_dir {
				if x.is_dir {
					Ordering::Less
				} else {
					Ordering::Greater
				}
			} else {
				lexical_sort::natural_lexical_cmp(&x.name.to_ascii_lowercase().to_string_lossy(), &y.name.to_ascii_lowercase().to_string_lossy())
			}
		);
		self.selection = None;
		self.selection_size = None;
		self.view = DataView::None;
		if let Some(selected_name) = selected_name {
			for (idx, c) in self.children.iter().enumerate() {
				if c.name == selected_name {
					let in_archive = self.path.get_archive_format();
					if let Ok(mut file_data) = self.path.load_file(&c.name) {
						let mut len = file_data.len();
						let mut decoded = auto_decode_full(file_data.to_mut(), in_archive);
						match decoded.data {
							DynData::Raw(ref mut raw_data) => {
								let msg = if decoded.steps_taken.is_empty() {
									format!("no steps taken; {}", decoded.error_msg)
								} else {
									format!("steps taken: {}; {}", decoded.steps_taken.join(" -> "), decoded.error_msg)
								};
								self.view = DataView::new_raw(raw_data, msg);
								len = raw_data.len();
							}
							DynData::Image(ref img) =>{
								let mut frames = Vec::new();
								for frame in &img.frames {
									let egui_img = egui::ColorImage::from_rgba_unmultiplied([frame.width as usize, frame.height as usize], frame.as_rgba_bytes());
									frames.push((egui_img.clone(), ctx.load_texture("image", egui_img, TextureOptions::LINEAR), frame.og_fmt));
								}
								self.view = DataView::Image(frames);
							}
							DynData::Archive(arc) => {
								self.selection = None;
								self.path.append_archive(&c.name, arc);
								self.refresh(ctx);
								return;
							}
						}
						self.selection = Some((idx, decoded.data, decoded.steps_taken));
						self.selection_size = Some(len);
					}
					break;
				}
			}
		}
		if let ComplexPath::Physical(path, _) = &self.path {
			let mut last = LAST_ACCESSED_PATH.write().unwrap();
			if last.as_ref() != Some(path) {
				last.replace(path.clone());
				std::mem::drop(last);
				dirty_config();
			}
		}
	}

	pub fn select(&mut self, ctx: &Context, idx: usize) {
		if idx < self.children.len() {
			self.selection = Some((idx, DynData::Raw(FileData::Memory {buf: Box::new([])}), Vec::new()));
			self.refresh(ctx);
		} else {
			self.selection = None;
		}
	}
}

#[macro_export]
macro_rules! image16 {
	($path:expr) => {
		egui::Image::new(egui::include_image!($path)).fit_to_exact_size(egui::vec2(16.0, 16.0))
	};
}

#[macro_export]
macro_rules! icon_button {
	($path:expr) => {
		egui::Button::image(crate::image16!($path))
	};
	($path:expr, $text:expr) => {
		egui::Button::image_and_text(crate::image16!($path), $text)
	};
}

struct ExplorerTabViewer<'a> {
	ctx: &'a Context,
	tab_to_add: &'a mut Option<(Option<(SurfaceIndex, NodeIndex)>, ExplorerTab)>
}

impl<'a> TabViewer for ExplorerTabViewer<'a> {
	type Tab = ExplorerTab;

	fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
		true
	}

	fn on_add(&mut self, surface: SurfaceIndex, node: NodeIndex) {
		let last = LAST_ACCESSED_PATH.read().unwrap().clone();
		if let Some(last) = last {
			*self.tab_to_add = Some((Some((surface, node)), ExplorerTab::new(self.ctx, last)));
		}
	}

	fn scroll_bars(&self, _: &Self::Tab) -> [bool; 2] {
		[false, false]
	}

	fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
		if let Some(x) = tab.path.file_name() {
			x.to_string_lossy().into()
		} else {
			tab.path.to_str().into()
		}
	}

	fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
		egui::Id::new(tab.id)
	}

	fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
		if let Some(task) = &mut tab.batch_task {
			if task.show(ui.ctx()) {
				tab.batch_task = None;
			}
		}
		let mut new_selection = None;
		if ui.input_mut(|inp| inp.consume_key(Modifiers::CTRL | Modifiers::SHIFT, Key::Tab)) {
			if let Some((selected_idx, ..)) = tab.selection {
				if selected_idx == 0 {
					new_selection = Some(tab.children.len() - 1);
				} else {
					new_selection = Some(selected_idx - 1);
				}
			} else if tab.children.len() != 0 {
				new_selection = Some(tab.children.len() - 1);
			}
		} else if ui.input_mut(|inp| inp.consume_key(Modifiers::CTRL, Key::Tab)) {
			if let Some((selected_idx, ..)) = tab.selection {
				if selected_idx >= tab.children.len() - 1 {
					new_selection = Some(0);
				} else {
					new_selection = Some(selected_idx + 1);
				}
			} else if tab.children.len() != 0 {
				new_selection = Some(0);
			}
		}
		ui.vertical_centered_justified(|ui| {
			ui.horizontal(|ui| {
				if ui.add_enabled(tab.past.len() != 0, icon_button!("icons/go-previous.svg")).clicked() {
					let new_path = tab.past.pop().unwrap();
					if new_path.can_iterate() {
						tab.selection = None;
						tab.future.push(std::mem::replace(&mut tab.path, new_path));
						tab.refresh(ui.ctx());
						log!("entered '{}'", tab.path.file_name().map_or_else(|| tab.path.to_str().into_owned(), |x| x.to_string_lossy().into_owned()));
					} else {
						log!("cannot read directory '{}'", new_path.file_name().unwrap_or_default().to_string_lossy());
					}
				}
				if ui.add_enabled(tab.future.len() != 0, icon_button!("icons/go-next.svg")).clicked() {
					let new_path = tab.future.pop().unwrap();
					if new_path.can_iterate() {
						tab.selection = None;
						tab.past.push(std::mem::replace(&mut tab.path, new_path));
						tab.refresh(ui.ctx());
						log!("entered '{}'", tab.path.file_name().map_or_else(|| tab.path.to_str().into_owned(), |x| x.to_string_lossy().into_owned()));
					} else {
						log!("cannot read directory '{}'", new_path.file_name().unwrap_or_default().to_string_lossy());
					}
				}
				{
					let parent = tab.path.parent();
					if ui.add_enabled(parent.is_some(), icon_button!("icons/go-up.svg")).clicked() {
						let new_path = parent.unwrap();
						if new_path.can_iterate() {
							tab.selection = None;
							tab.past.push(std::mem::replace(&mut tab.path, new_path));
							tab.future.clear();
							tab.refresh(ui.ctx());
							log!("entered '{}'", tab.path.file_name().map_or_else(|| tab.path.to_str().into_owned(), |x| x.to_string_lossy().into_owned()));
						} else {
							log!("cannot read directory '{}'", new_path.file_name().unwrap_or_default().to_string_lossy());
						}
					}
				}
				if ui.add(icon_button!("icons/view-refresh.svg")).clicked() {
					tab.refresh(ui.ctx());
					log!("refreshed '{}'", tab.path.file_name().map_or_else(|| tab.path.to_str().into_owned(), |x| x.to_string_lossy().into_owned()));
				}
				ui.separator();
				if let Some(home) = HOME_DIR.as_ref() {
					if let Ok(rest) = tab.path.to_str_stripping_dir_prefix(&home) {
						ui.label(format!("~/{}", rest));
					} else {
						ui.label(tab.path.to_str());
					}
				} else {
					ui.label(tab.path.to_str());
				}
				ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
					let mut bookmarks = BOOKMARKS.write().unwrap();
					let bookmark_list_response = ui.add_enabled(bookmarks.len() != 0, icon_button!("icons/bookmarks-bookmarked.svg"));
					let popup_id = ui.make_persistent_id("bookmark list popup");
					if bookmark_list_response.clicked() {
						ui.memory_mut(|mem| mem.toggle_popup(popup_id));
					}
					let mut index_to_remove = None;
					popup::popup_below_widget(
						ui,
						popup_id,
						&bookmark_list_response,
						PopupCloseBehavior::CloseOnClickOutside,
						|ui| {
							ui.spacing_mut().button_padding.y = 0.0;
							let row_height = ui.spacing().interact_size.y;
							ScrollArea::vertical().show_rows(ui, row_height, bookmarks.len(), |ui, row_range| {
								Grid::new("bookmark list grid")
									.num_columns(1)
									.max_col_width(480.0)
									.striped(true)
									.start_row(row_range.start)
									.show(ui, |ui| {
										ui.set_width(480.0);
										ui.set_height(320.0);
										for (index, bookmark) in bookmarks.iter().enumerate().rev() {
											ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
												if ui.add(icon_button!("icons/edit-delete.svg")).clicked() {
													index_to_remove = Some(index);
													if bookmarks.len() == 1 {
														ui.memory_mut(|mem| mem.close_popup());
													}
												}
												ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
													if ui.add(Button::new(bookmark.to_string_lossy()).frame(false).wrap()).clicked() {
														if bookmark.is_dir() {
															tab.path = ComplexPath::Physical(bookmark.clone(), true);
															tab.refresh(ui.ctx());
														} else {
															log!("invalid bookmark");
														}
													}
												});
											});
											ui.end_row();
										}
									})
							});
						}
					);
					if let Some(index) = index_to_remove {
						bookmarks.remove(index);
						dirty_config();
					}
					if let ComplexPath::Physical(path, _) = &tab.path {
						if let Some(idx) = bookmarks.iter().position(|x| x == path) {
							if ui.add(icon_button!("icons/bookmark-remove.svg")).clicked() {
								bookmarks.remove(idx);
								dirty_config();
							} else if idx != bookmarks.len() - 1 {
								let p = bookmarks.remove(idx);
								bookmarks.push(p);
								dirty_config();
							}
						} else {
							if ui.add(icon_button!("icons/bookmark-new.svg")).clicked() {
								bookmarks.push(path.clone());
								dirty_config();
							}
						}
					}
					ui.separator();
				});
			});
			ui.separator();
			if tab.children.len() == 0 {
				ui.label("Empty");
				return;
			}
			let area = ui.available_rect_before_wrap();
			const LIST_WIDTH: f32 = 240.0;
			const ICON_WIDTH: f32 = 20.0;
			let line_height = ui.text_style_height(&TextStyle::Body);
			let info_block_height = line_height * 2.0;
			const SEPARATOR_HEIGHT: f32 = 2.0;
			let list_height = area.height() - info_block_height - 8.0 - SEPARATOR_HEIGHT;
			// file list
			ui.allocate_new_ui(UiBuilder::new().max_rect(Rect::from_min_size(
				Pos2::new(area.min.x, area.min.y),
				Vec2::new(LIST_WIDTH, list_height)
			)), |ui| {
				ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
					ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
						ui.label(tab.path.get_archive_format().unwrap_or("folder"));
						ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
							let folder_name = tab.path.file_name().map_or_else(|| tab.path.to_str(), |x| Cow::Owned(x.to_string_lossy().into_owned()));
							ui.add(Label::new(folder_name).truncate());
						});
					});
					ui.vertical_centered_justified(|ui| {
						if ui.small_button("Batch Decode...").clicked() {
							let mut task = BatchDecode::new(tab.path.clone());
							if !task.show(ui.ctx()) {
								tab.batch_task = Some(task);
							}
						}
					});
					ui.separator();
					ui.spacing_mut().button_padding.y = 0.0;
					let row_height = ui.spacing().interact_size.y;
					ScrollArea::vertical().show_rows(ui, row_height, tab.children.len(), |ui, row_range| {
						Grid::new("file list grid")
							.num_columns(1)
							.max_col_width(LIST_WIDTH)
							.striped(true)
							.start_row(row_range.start)
							.show(ui, |ui| {
								let selection_idx = tab.selection.as_ref().map(|x| x.0);
								for i in row_range {
									ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
										if ui.add(
											if tab.children[i].is_dir {
												icon_button!("icons/document-open-folder.svg", tab.children[i].name.to_string_lossy())
											} else {
												Button::new(tab.children[i].name.to_string_lossy())
											}.frame(false).selected(selection_idx == Some(i))
										).clicked() {
											new_selection = Some(i);
										}
									});
									ui.end_row();
								}
							});
					});
				});
			});
			// info block icon for file type
			ui.allocate_new_ui(UiBuilder::new().max_rect(Rect::from_min_size(
				Pos2::new(area.min.x, area.max.y - info_block_height),
				Vec2::new(16.0, info_block_height)
			)), |ui| {
				ui.centered_and_justified(|ui| {
					match tab.view {
						DataView::None => {}
						DataView::Image(..) => {
							ui.add(image16!("icons/image-x-generic.svg"));
						}
						DataView::Raw {..} => {
							ui.add(image16!("icons/application-octet-stream.svg"));
						}
					}
				});
			});
			// info block file name
			ui.allocate_new_ui(UiBuilder::new().max_rect(Rect::from_min_size(
				Pos2::new(area.min.x + ICON_WIDTH, area.max.y - info_block_height),
				Vec2::new(LIST_WIDTH - ICON_WIDTH, line_height)
			)), |ui| {
				ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
					if let Some((idx, ..)) = tab.selection {
						ui.add(Label::new(tab.children[idx].name.to_string_lossy()).wrap_mode(TextWrapMode::Truncate));
					}
				});
			});
			// info block file size
			ui.allocate_new_ui(UiBuilder::new().max_rect(Rect::from_min_size(
				Pos2::new(area.min.x + ICON_WIDTH, area.max.y - line_height),
				Vec2::new(LIST_WIDTH - ICON_WIDTH, line_height)
			)), |ui| {
				ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
					if let Some(size) = tab.selection_size {
						let formatted = if size < 1024 {
							format!("{} bytes", size)
						} else if size < 1024 * 1024 {
							format!("{}.{} KiB", size / 1024, size * 10 / 1024 % 10)
						} else if size < 1024 * 1024 * 1024 {
							format!("{}.{} MiB", size / (1024 * 1024), size * 10 / (1024 * 1024) % 10)
						} else {
							format!("{}.{} GiB", size / (1024 * 1024 * 1024), size * 10 / (1024 * 1024 * 1024) % 10)
						};
						ui.add(Label::new(formatted).wrap_mode(TextWrapMode::Truncate));
					}
					if let Some((.., steps_taken)) = &tab.selection {
						ui.separator();
						if steps_taken.is_empty() {
							ui.label("no steps taken");
						} else {
							ui.label(steps_taken.join(" -> "));
						}
					}
				});
			});
			// separator between file list and info block
			ui.allocate_new_ui(UiBuilder::new().max_rect(Rect::from_min_size(
				Pos2::new(area.min.x, area.min.y + list_height + 3.0),
				Vec2::new(LIST_WIDTH, 6.0)
			)), |ui| {
				ui.add_sized(Vec2::new(LIST_WIDTH, SEPARATOR_HEIGHT), Separator::default().horizontal());
			});
			// actual file view
			ui.allocate_new_ui(UiBuilder::new().max_rect(Rect::from_min_size(
				Pos2::new(area.min.x + LIST_WIDTH, area.min.y),
				Vec2::new(area.width() - LIST_WIDTH, area.height())
			)), |ui| {
				ui.horizontal_centered(|ui| {
					ui.separator();
					tab.view.ui(ui);
				})
			});
		});
		if let Some(idx) = new_selection {
			if tab.children[idx].is_dir {
				let new_path = tab.path.join_dir(&tab.children[idx].name);
				if new_path.can_iterate() {
					tab.selection = None;
					tab.past.push(std::mem::replace(&mut tab.path, new_path));
					tab.future.clear();
					tab.refresh(ui.ctx());
					log!("entered '{}'", tab.path.file_name().map_or_else(|| tab.path.to_str().into_owned(), |x| x.to_string_lossy().into_owned()));
				} else {
					log!("cannot read directory '{}'", tab.children[idx].name.to_string_lossy());
				}
			} else {
				tab.select(ui.ctx(), idx);
			}
		}
	}
}

struct App {
	dock_state: DockState<ExplorerTab>
}

impl eframe::App for App {
	fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
		let mut tab_to_add = None;
		TopBottomPanel::bottom("bottom panel").show(ctx, |ui| {
			ui.horizontal_centered(|ui| {
				if ui.small_button("Open Folder...").clicked() {
					if let Some(folder) = FileDialog::new().pick_folder() {
						if folder.is_dir() {
							tab_to_add = Some((None, ExplorerTab::new(ctx, folder)));
						}
					}
				}
				ui.separator();
				ui.label(get_last_log());
			});
		});
		CentralPanel::default().show(ctx, |ui| {
			DockArea::new(&mut self.dock_state)
				.style({
					let mut style = egui_dock::Style::from_egui(ctx.style().as_ref());
					style.buttons.add_tab_align = TabAddAlign::Left;
					style
				})
				.show_add_buttons(true)
				.window_bounds(ui.clip_rect())
				.show_inside(ui, &mut ExplorerTabViewer {ctx, tab_to_add: &mut tab_to_add});
			if let Some((specific, tab)) = tab_to_add {
				if let Some((surface, node)) = specific {
					self.dock_state[surface][node].append_tab(tab);
				} else {
					if let Some(root_node) = self.dock_state.main_surface_mut().root_node_mut() {
						root_node.append_tab(tab);
					} else {
						self.dock_state.add_window(vec![tab]);
					}
				}
			}
		});
		if IS_CONFIG_DIRTY.load(atomic::Ordering::Acquire) {
			write_config();
		}
	}
}

fn main() {
	read_config();
	eframe::run_native(
		"Kidfile",
		eframe::NativeOptions {
			viewport: ViewportBuilder::default()
				.with_title("Kidfile Explorer")
				.with_inner_size(vec2(1024.0, 640.0)),
			..Default::default()
		},
		Box::new(|cc| {
			egui_extras::install_image_loaders(&cc.egui_ctx);
			cc.egui_ctx.add_font(FontInsert::new(
				"NotoSansJP",
				FontData::from_static(include_bytes!("fonts/NotoSansJP-VariableFont_wght.ttf")),
				vec![InsertFontFamily {family: FontFamily::Proportional, priority: FontPriority::Highest}]
			));
			cc.egui_ctx.add_font(FontInsert::new(
				"NotoSansMonoCJKjp",
				FontData::from_static(include_bytes!("fonts/NotoSansMonoCJKjp-Regular.otf")),
				vec![InsertFontFamily {family: FontFamily::Monospace, priority: FontPriority::Highest}]
			));
			cc.egui_ctx.style_mut(|style| {
				style.spacing.button_padding = vec2(8.0, 8.0);
			});
			let mut tabs: Vec<ExplorerTab> = Vec::new();
			let last = LAST_ACCESSED_PATH.read().unwrap().clone();
			if let Some(last) = last {
				tabs.push(ExplorerTab::new(&cc.egui_ctx, last));
			}
			Ok(Box::new(App {
				dock_state: DockState::new(tabs)
			}))
		})
	).unwrap();
}
