use egui::{Align, ColorImage, Layout, ScrollArea, TextEdit, TextureHandle, Ui, Vec2};
use kidfile::{file_data::FileData, image::PixelFormat};

use crate::icon_button;

pub enum DataView {
	None,
	Raw {
		hex: String,
		error_msg: String,
		reset_view: bool
	},
	Image(Vec<(ColorImage, TextureHandle, PixelFormat)>)
}

impl DataView {
	pub fn new_raw(file_data: &mut FileData, error_msg: String) -> Self {
		const MAX_LEN: usize = 2048;
		const BYTES_IN_LINE: usize = 16;
		let mut buf = unsafe {Box::new_uninit_slice(file_data.len().min(MAX_LEN)).assume_init()};
		if file_data.read_chunk_exact(&mut buf, 0).is_ok() {
			let mut hex = String::new();
			for (i, line_chunk) in buf.chunks(BYTES_IN_LINE).enumerate() {
				hex += &format!("{:0>4X} | ", i * BYTES_IN_LINE);
				for i in 0..BYTES_IN_LINE {
					if let Some(x) = line_chunk.get(i) {
						hex += &format!("{:0>2X} ", x);
					} else {
						hex += "-- "
					}
				}
				hex += "| ";
				for b in line_chunk {
					hex.push(if let Some(c) = char::from_u32(*b as u32) {
						if c.is_ascii_control() {
							'.'
						} else {
							c
						}
					} else {
						'.'
					});
				}
				hex.push('\n');
			}
			if buf.len() < file_data.len() {
				hex += &format!("file truncated at 0x{MAX_LEN:X}/{MAX_LEN}, full size is 0x{:X}/{}", file_data.len(), file_data.len());
			} else {
				hex += &format!("file displayed in full, size is 0x{:X}/{}", file_data.len(), file_data.len());
			}
			Self::Raw {hex, error_msg, reset_view: true}
		} else {
			Self::Raw {hex: "<error reading file>".into(), error_msg, reset_view: true}
		}
	}

	pub fn ui(&mut self, ui: &mut Ui) {
		match self {
			Self::None => {}
			Self::Raw {hex, error_msg, reset_view} => {
				ui.vertical(|ui| {
					ui.label(error_msg.as_str());
					ScrollArea::both().id_salt("hex").show(ui, |ui| {
						if *reset_view {
							ui.scroll_to_cursor(Some(Align::Min));
							*reset_view = false;
						}
						ui.centered_and_justified(|ui| ui.add_enabled(false,
							TextEdit::multiline(&mut hex.clone())
								.id_salt("hex")
								.code_editor()
								.cursor_at_end(false)
						))
					});
				});
			}
			Self::Image(frames) => {
				const OTHER_FRAMES_WIDTH: f32 = 128.0;
				ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
					if frames.len() > 1 {
						ui.allocate_ui(Vec2::new(OTHER_FRAMES_WIDTH, ui.available_height()), |ui| {
							ScrollArea::vertical().show(ui, |ui| {
								ui.set_min_width(OTHER_FRAMES_WIDTH);
								ui.vertical(|ui| {
									for (frame_idx, (egui_img, tex, fmt)) in frames.iter().enumerate().skip(1) {
										if frame_idx >= 2 {
											ui.separator();
										}
										ui.horizontal(|ui| {
											ui.label(format!("#{frame_idx}, {}x{}, {}", tex.size()[0], tex.size()[1], fmt));
											if ui.add(icon_button!("icons/edit-copy.svg").small()).on_hover_text("Copy to clipboard").clicked() {
												ui.ctx().copy_image(egui_img.clone());
											}
										});
										ui.add(egui::Image::new(tex).fit_to_exact_size(Vec2::new(OTHER_FRAMES_WIDTH, OTHER_FRAMES_WIDTH)));
									}
								});
							});
						});
					}
					ui.vertical(|ui| {
						ui.horizontal(|ui| {
							if frames.len() > 1 {
								ui.label(format!("#0, {}x{}, {} ({} frames)", frames[0].0.width(), frames[0].0.height(), frames[0].2, frames.len()));
							} else {
								ui.label(format!("{}x{}, {}", frames[0].0.width(), frames[0].0.height(), frames[0].2));
							}
							if ui.add(icon_button!("icons/edit-copy.svg").small()).on_hover_text("Copy to clipboard").clicked() {
								ui.ctx().copy_image(frames[0].0.clone());
							}
						});
						ui.centered_and_justified(|ui| {
							ui.add(egui::Image::new(&frames[0].1).fit_to_exact_size(ui.available_size()));
						});
					});
					ui.separator();
				});
			}
		}
	}
}