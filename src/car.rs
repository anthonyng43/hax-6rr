use crate::*;
use anyhow::Result;
use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions, Vec2, Color32, Stroke};
use std::{io::Cursor, u32};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use strum::{EnumIter, IntoEnumIterator};
use url::Url;

pub struct CarMenu {
	pub play_count: u32,
	pub odometer: u32,
	pub vs_cool_or_wild: i32,
	pub vs_smooth_or_rough: i32,
	pub vs_play_count: u32,
	pub odometer_buf: String,
	pub vs_star_count: u32,
	pub vs_star_count_buf: String,
	pub vs_gold_medal: u32,
	pub vs_silver_medal: u32,
	pub vs_bronze_medal: u32,
	pub vs_plain_medal: u32,
	pub gold_medal_buf: String,
	pub silver_medal_buf: String,
	pub bronze_medal_buf: String,
	pub plain_medal_buf: String,
}

impl CarMenu {
	pub fn update(
		&mut self,
		ui: &mut egui::Ui,
		runtime: &tokio::runtime::Runtime,
		server: &Url,
		car: &mut wm::Car,
		car_settings: &mut wm::CarSetting,
		car_items: &[wm::CarItem],
		glb_enabled: &mut bool,
		custom: &mut bool,
		custom_color: &mut bool,
		force: &mut bool,
	) {
		runtime.block_on(async {
			if let Some(selected_car) = wm::Cars::from_u32(car.visual_model()) {
				let have_dressup = selected_car.have_dress_up();
				let have_limited_dressup = selected_car.limited_dress_up();
				
				let mut save_clicked = false;
				let mut update_clicked = false;
			
				ui.horizontal(|ui| {
					if ui.button("Save").clicked() {
						save_clicked = true;
					}
			
					if have_dressup {
						if ui.button("Update Car Dressup").clicked() {
							update_clicked = true;
						}
					}
				});

				if save_clicked {
					_ = save_car(
						server,
						car,
						car_settings,
						self.play_count,
						self.odometer_buf
							.parse()
							.unwrap_or(self.odometer),
						self.vs_star_count_buf
							.parse()
							.unwrap_or(self.vs_star_count),
						self.vs_cool_or_wild,
						self.vs_smooth_or_rough,
						self.vs_play_count,
						self.gold_medal_buf
							.parse()
							.unwrap_or(self.vs_gold_medal),
						self.silver_medal_buf
							.parse()
							.unwrap_or(self.vs_silver_medal),
						self.bronze_medal_buf
							.parse()
							.unwrap_or(self.vs_bronze_medal),
						self.plain_medal_buf
							.parse()
							.unwrap_or(self.vs_plain_medal),
					)
					.await;
				}
			
				if update_clicked {
					_ = update_car(
						server,
						car,
						car_settings
					)
					.await;
				}

				egui::Grid::new("CarGrid").num_columns(2).show(ui, |ui| {
					set_car_class(ui, car);
					set_region_and_country(ui, car, glb_enabled);
					
					if have_dressup {
						if have_limited_dressup {
							set_aero_set(ui, car, car_items);
							set_aero_mirror(ui, car, car_items);
							set_bonnet(ui, car, car_items);
							set_number_plate_frame(ui, car, car_items);
							set_wing(ui, car, car_items, custom);
						} else {
							set_aero_set(ui, car, car_items);
							set_aero_mirror(ui, car, car_items);
							set_bonnet(ui, car, car_items);
							set_number_plate_frame(ui, car, car_items);
							set_neon(ui, car, car_items);
							set_trunk(ui, car, car_items);
							set_wing(ui, car, car_items, custom);
						}
					}

					set_color(ui, car, car_items, custom_color);
					set_wheel(ui, car, car_items, force);
					set_vs_grade(ui, car, car_items);
					set_rival_marker(ui, car, car_items);
					set_window_deco(ui, car, car_items);
					set_volume(ui, car_settings);
					set_bgm(ui, car_settings, car_items);
					set_meter(ui, car_settings, car_items);
					set_name_frame(ui, car, car_items);
					set_nameplate(ui, car_settings, car_items);
					set_terminal_background(ui, car_settings, car_items);

					ui.label("Navigation Map");
					ui.add(egui::Checkbox::without_text(&mut car_settings.navigation_map));
					ui.end_row();

					ui.label("Retire");
					ui.add(egui::Checkbox::without_text(&mut car_settings.retire));
					ui.end_row();

					ui.label("Manual Transmission");
					ui.add(egui::Checkbox::without_text(&mut car_settings.transmission));
					ui.end_row();

					ui.label("Third Person");
					ui.add(egui::Checkbox::without_text(&mut car_settings.view));
					ui.end_row();

					ui.label("Title");
					ui.add(egui::TextEdit::singleline(&mut car.title));
					ui.end_row();

					ui.label("Car Plate Number(1-9999 Only!, 0 for default)");
					let mut plate_number_string = car.plate_number.to_string();
					ui.add(egui::TextEdit::singleline(&mut plate_number_string));
					car.plate_number = match plate_number_string.parse::<u32>() {
						Ok(num) => num.clamp(0, 9999),
						Err(_) => car.plate_number,
					};
					ui.end_row();

					ui.label("Odometer");
					ui.add(egui::TextEdit::singleline(&mut self.odometer_buf));
					ui.end_row();

					ui.label("Star Count");
					ui.add(egui::TextEdit::singleline(&mut self.vs_star_count_buf));
					ui.end_row();

					ui.label("Gold Vs Medal");
					ui.add(egui::TextEdit::singleline(&mut self.gold_medal_buf));
					ui.end_row();

					ui.label("Silver Vs Medal");
					ui.add(egui::TextEdit::singleline(&mut self.silver_medal_buf));
					ui.end_row();

					ui.label("Bronze Vs Medal");
					ui.add(egui::TextEdit::singleline(&mut self.bronze_medal_buf));
					ui.end_row();

					ui.label("Plain Vs Medal");
					ui.add(egui::TextEdit::singleline(&mut self.plain_medal_buf));
					ui.end_row();
					ui.end_row();
					
					aura_axis(ui, &mut self.vs_cool_or_wild, &mut self.vs_smooth_or_rough);
					car_aura(car, self.odometer, self.vs_cool_or_wild, self.vs_smooth_or_rough);
				});
			}
		});
	}

	pub fn back(&mut self) -> bool {
		true
	}
}

fn set_volume(ui: &mut egui::Ui, car_settings: &mut wm::CarSetting) {
	#[derive(FromPrimitive, ToPrimitive, EnumIter)]
	enum Volume {
		Muted = 0,
		Minimum = 1,
		Medium = 2,
		Maximum = 3,
	}

	impl ToString for Volume {
		fn to_string(&self) -> String {
			match self {
				Volume::Muted => String::from("Muted"),
				Volume::Minimum => String::from("Minimum"),
				Volume::Medium => String::from("Medium"),
				Volume::Maximum => String::from("Maximum"),
			}
		}
	}

	ui.label("Volume");
	egui::ComboBox::from_id_source("VolumeComboBox")
		.selected_text(
			Volume::from_u32(car_settings.volume)
				.unwrap_or(Volume::Medium)
				.to_string(),
		)
		.show_ui(ui, |ui| {
			for volume in Volume::iter() {
				ui.selectable_value(
					&mut car_settings.volume,
					volume.to_u32().unwrap_or(2),
					volume.to_string(),
				);
			}
		});
	ui.end_row();
}

fn set_bgm(ui: &mut egui::Ui, car_settings: &mut wm::CarSetting, car_items: &[wm::CarItem]) {
	ui.label("Bgm");

	let selected = match wm::Bgms::from_u32(car_settings.bgm) {
		Some(bgm) => bgm.to_string(),
		None => String::from("WMMT 6/6R/6RR"),
	};
	egui::ComboBox::from_id_source("BgmComboBox")
		.selected_text(selected)
		.show_ui(ui, |ui| {
			ui.selectable_value(&mut car_settings.bgm, 0, "WMMT 6/6R/6RR");
			for bgm in wm::Bgms::iter() {
				if car_items
					.iter()
					.filter(|item| {
						item.category == wm::ItemCategory::CatBgm.into()
							&& Some(item.item_id) == bgm.to_u32()
					})
					.collect::<Vec<_>>()
					.len() == 1
				{
					ui.selectable_value(
						&mut car_settings.bgm,
						bgm.to_u32().unwrap_or(0),
						bgm.to_string(),
					);
				}
			}
		});
	ui.end_row();
}

fn set_meter(ui: &mut egui::Ui, car_settings: &mut wm::CarSetting, car_items: &[wm::CarItem]) {
	ui.label("Meter");

	let selected = match wm::Meters::from_u32(car_settings.meter) {
		Some(meter) => meter.to_string(),
		None => String::from("Stock"),
	};
	egui::ComboBox::from_id_source("MeterComboBox")
		.selected_text(selected)
		.show_ui(ui, |ui| {
			ui.selectable_value(&mut car_settings.meter, 0, "Stock");
			for meter in wm::Meters::iter() {
				if car_items
					.iter()
					.filter(|item| {
						item.category == wm::ItemCategory::CatMeter.into()
							&& Some(item.item_id) == meter.to_u32()
					})
					.collect::<Vec<_>>()
					.len() == 1
				{
					ui.selectable_value(
						&mut car_settings.meter,
						meter.to_u32().unwrap_or(0),
						meter.to_string(),
					);
				}
			}
		});
	ui.end_row();
}

fn set_nameplate(ui: &mut egui::Ui, car_settings: &mut wm::CarSetting, car_items: &[wm::CarItem]) {
	ui.label("Nameplate");

	let selected = match wm::Nameplates::from_u32(car_settings.nameplate) {
		Some(nameplate) => nameplate.to_string(),
		None => String::from("Stock"),
	};
	egui::ComboBox::from_id_source("NameplateComboBox")
		.selected_text(selected)
		.show_ui(ui, |ui| {
			ui.selectable_value(&mut car_settings.nameplate, 0, "Stock");
			for nameplate in wm::Nameplates::iter() {
				if !car_items
					.iter()
					.filter(|item| {
						item.category == wm::ItemCategory::CatNamePlate.into()
							&& Some(item.item_id) == nameplate.to_u32()
					})
					.collect::<Vec<_>>()
					.is_empty()
				{
					ui.selectable_value(
						&mut car_settings.nameplate,
						nameplate.to_u32().unwrap_or(0),
						nameplate.to_string(),
					);
				}
			}
		});
	ui.end_row();
}

fn set_car_class(ui: &mut egui::Ui, car: &mut wm::Car) {
	const CLASSES: &[&str] = &["C", "B", "A", "S", "SS", "SSS", "SSSS", "SSSSS"];

	fn get_class(class: u32) -> String {
		if class == 1 {
			return String::from("N");
		} else if class >= 74 {
			return String::from("SSSSSS");
		}

		let class = class as usize - 2;
		let numbers: Vec<u32> = (1..=9).rev().collect();
		format!("{}{}", CLASSES[class / 9], numbers[class % 9])
	}

	ui.label("Class");
	egui::ComboBox::from_id_source("ClassComboBox")
		.selected_text(get_class(car.level))
		.show_ui(ui, |ui| {
			for class in 1..=(CLASSES.len() as u32 * 9 + 2) {
				ui.selectable_value(&mut car.level, class, get_class(class));
			}
		});
	ui.end_row();
}

// Thanks Brogamer for getting color name list's
fn set_color(ui: &mut egui::Ui, car: &mut wm::Car, car_items: &[wm::CarItem], custom_color: &mut bool) {
    ui.horizontal(|ui| {
        ui.label("Car Color");
		if car_items.iter().any(|item| {
			item.category == wm::ItemCategory::CatCustomColor.into()
				&& Some(item.item_id) == Some(1)
		}) {
		ui.checkbox(custom_color, "Custom Color");
		}
    });

    if *custom_color {
        if let Some(selected_car) = wm::Cars::from_u32(car.visual_model()) {
            let custom_colors = selected_car.custom_colors();

            if !custom_colors.is_empty() {
                let selected_color_name = &custom_colors[car.custom_color as usize].name;

				egui::ComboBox::from_id_source("CarCustomColorComboBox")
					.selected_text(selected_color_name)
					.show_ui(ui, |ui| {
						for (i, color) in custom_colors.iter().enumerate() {
							if car_items.iter().any(|item| {
								item.category == wm::ItemCategory::CatCustomColor.into()
									&& Some(item.item_id) == Some(i as u32)
							}) {
								ui.selectable_value(
									&mut car.custom_color,
									i as u32,
									&color.name,
								);
							}
						}
					});
			}
        }
    } else {
        car.custom_color = 0;

        if let Some(selected_car) = wm::Cars::from_u32(car.visual_model()) {
            let default_colors = selected_car.default_colors();

            if !default_colors.is_empty() {
                let selected_color_name = &default_colors[car.default_color() as usize].name;

                egui::ComboBox::from_id_source("DefaultColorComboBox")
                    .selected_text(selected_color_name)
                    .show_ui(ui, |ui| {
                        for (i, color) in default_colors.iter().enumerate() {
                            ui.selectable_value(
                                &mut car.default_color,
                                Some(i as u32),
                                &color.name,
                            );
                        }
                    });
            }
        }
    }
    ui.end_row();
}

fn set_region_and_country(ui: &mut egui::Ui, car: &mut wm::Car, glb_enabled: &mut bool) {
    fn get_region_list(region: Option<u32>) -> String {
        if let Some(region_id) = region {
            if let Some(jpn) = wm::Jpn::from_u32(region_id) {
                return jpn.to_string();
            }
        }
        String::from("Not Valid Selection")
    }

    fn get_glb_list(glb: Option<u32>) -> String {
        if let Some(glb_id) = glb {
            if let Some(glb) = wm::Glb::from_u32(glb_id) {
                return glb.to_string();
            }
        }
        String::from("Not Valid Selection")
    }

    ui.horizontal(|ui| {
        ui.label("Region");
		ui.checkbox(glb_enabled, "GLB");
	});

	if *glb_enabled {
		car.country = Some(String::from("GLB"));

		egui::ComboBox::from_id_source("GlbComboBox")
			.selected_text(get_glb_list(car.region_id))
			.show_ui(ui, |ui| {
				for glb_id in 1..=47 {
					let glb_option = Some(glb_id);
					ui.selectable_value(&mut car.region_id, glb_option, get_glb_list(glb_option));
				}
			});
	} else {
		car.country = Some(String::from("JPN"));

		egui::ComboBox::from_id_source("RegionIdComboBox")
			.selected_text(get_region_list(car.region_id))
			.show_ui(ui, |ui| {
				for region_id in 1..=47 {
					let region_option = Some(region_id);
					ui.selectable_value(&mut car.region_id, region_option, get_region_list(region_option));
				}
			});
	}
    ui.end_row();
}

fn set_rival_marker(ui: &mut egui::Ui, car: &mut wm::Car, car_items: &[wm::CarItem]) {
    ui.label("Rival Marker");

    let selected = match car.rival_marker {
        Some(rival_marker_value) => match wm::RivalMarker::from_u32(rival_marker_value) {
            Some(rivalmarker) => rivalmarker.to_string(),
            None => String::from("Stock"),
        },
        None => String::from("Stock"),
    };

    egui::ComboBox::from_id_source("RivalMarkerComboBox")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut car.rival_marker, Some(0), "Stock");
            for rivalmarker in wm::RivalMarker::iter() {
                if car_items.iter().any(|item| {
                    item.category == wm::ItemCategory::CatRivalMarker.into()
                        && Some(item.item_id) == rivalmarker.to_u32()
                }) {
                    ui.selectable_value(
                        &mut car.rival_marker,
                        rivalmarker.to_u32(),
                        rivalmarker.to_string(),
                    );
                }
            }
        });
    ui.end_row();
}

fn set_window_deco(ui: &mut egui::Ui, car: &mut wm::Car, car_items: &[wm::CarItem]) {
	ui.label("Window Decoration");

	let selected = match car.window_decoration {
        Some(deco_value) => match wm::TeamDeco::from_u32(deco_value) {
            Some(windowdecoration) => windowdecoration.to_string(),
            None => String::from("Stock"),
        },
		None => String::from("Stock"),
    };
	egui::ComboBox::from_id_source("WindowDecorationComboBox")
		.selected_text(selected)
		.show_ui(ui, |ui| {
			ui.selectable_value(&mut car.window_decoration, Some(0), "Stock");
			for windowdecoration in wm::TeamDeco::iter() {
				if !car_items
					.iter()
					.filter(|item| {
						item.category == wm::ItemCategory::CatWindowDecoration.into()
							&& Some(item.item_id) == windowdecoration.to_u32()
					})
					.collect::<Vec<_>>()
					.is_empty()
				{
					ui.selectable_value(
						&mut car.window_decoration,
						windowdecoration.to_u32(),
						windowdecoration.to_string(),
					);
				}
			}
		});
	ui.end_row();
}

fn set_terminal_background(ui: &mut egui::Ui, car_settings: &mut wm::CarSetting, car_items: &[wm::CarItem]) {
	ui.label("Terminal Background");

	let selected = match wm::TerminalBackground::from_u32(car_settings.terminal_background) {
		Some(terminalbackground) => terminalbackground.to_string(),
		None => String::from("Stock"),
	};
	egui::ComboBox::from_id_source("TerminalBackgroundComboBox")
		.selected_text(selected)
		.show_ui(ui, |ui| {
			ui.selectable_value(&mut car_settings.terminal_background, 0, "Stock");
			for terminalbackground in wm::TerminalBackground::iter() {
				if !car_items
					.iter()
					.filter(|item| {
						item.category == wm::ItemCategory::CatTerminalBackground.into()
							&& Some(item.item_id) == terminalbackground.to_u32()
					})
					.collect::<Vec<_>>()
					.is_empty()
				{
					ui.selectable_value(
						&mut car_settings.terminal_background,
						terminalbackground.to_u32().unwrap_or(0),
						terminalbackground.to_string(),
					);
				}
			}
		});
	ui.end_row();
}

fn set_vs_grade(ui: &mut egui::Ui, car: &mut wm::Car, car_items: &[wm::CarItem]) {
    fn get_vs_aura(id: usize) -> String {
        if id > 90 {
            match id {
                100 => String::from("Anniversary"),
                101 => String::from("Halloween"),
                _ => panic!("Unknown vs aura"),
            }
        } else {
            format!("{} {}", wm::VS_GRADES[(id - 1) / 3], ((id - 1) % 3) + 1)
        }
    }

    ui.label("Vs Aura");
    let selected = match car.aura_motif {
        Some(aura_motif_value) if (1..=(wm::VS_GRADES.len() * 3) as u32).contains(&aura_motif_value) => {
            get_vs_aura(aura_motif_value as usize)
        }
        _ => String::from("No Vs Grade"),
    };

    egui::ComboBox::from_id_source("VsAuraComboBox")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut car.aura_motif, Some(0), "No Vs Grade");

            for grades in 1..=(wm::VS_GRADES.len() * 3) {
                if car_items.iter().any(|item| {
                    item.category == wm::ItemCategory::CatAuraMotif.into()
                        && Some(item.item_id) == grades.to_u32()
                }) {
                    ui.selectable_value(
                        &mut car.aura_motif,
                        grades.to_u32(),
                        get_vs_aura(grades).to_string(),
                    );
                }
            }

            let special_items = [
                (100, "Anniversary"),
                (101, "Halloween")
            ];

            for &(id, label) in &special_items {
                if car_items.iter().any(|item| {
                    item.category == wm::ItemCategory::CatAuraMotif.into()
                        && Some(item.item_id) == id.to_u32()
                }) {
                    ui.selectable_value(&mut car.aura_motif, Some(id), label);
                }
            }
        });
    ui.end_row();
}

fn set_name_frame(ui: &mut egui::Ui, car: &mut wm::Car, car_items: &[wm::CarItem]) {
    ui.label("Custom Frame");

    let selected = match car.custom_frame {
        Some(custom_frame_value) => match wm::CustomFrame::from_u32(custom_frame_value) {
            Some(customframe) => customframe.to_string(),
            None => String::from("Stock"),
        },
        None => String::from("Stock"),
    };

    egui::ComboBox::from_id_source("CustomFrameComboBox")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut car.custom_frame, Some(0), "Stock");
            for customframe in wm::CustomFrame::iter() {
                if car_items.iter().any(|item| {
                    item.category == wm::ItemCategory::CatCustomFrame.into()
                        && Some(item.item_id) == customframe.to_u32()
                }) {
                    ui.selectable_value(
                        &mut car.custom_frame,
                        customframe.to_u32(),
                        customframe.to_string(),
                    );
                }
            }
        });
    ui.end_row();
}

// thanks brogamer
fn car_aura(car: &mut wm::Car, odometer: u32, vs_cool_or_wild: i32, vs_smooth_or_rough: i32,) {
	let current_mileage = odometer;
	let offset = match current_mileage {
		0..=5000 => 0,
		5001..=10000 => 1,
		10001..=30000 => 2,
		30001..=100000 => 3,
		100001..=500000 => 4,
		500001..=1000000 => 5,
		1000001..=2000000 => 6,
		_ => 7,
	};
	car.aura = Some((544 + ((vs_cool_or_wild + 4) * 8 * 8 * 2) + ((vs_smooth_or_rough + 4) * 8) + offset) as u32);
}

const COLOR_IMAGE_BYTES: &[u8] = include_bytes!("color.png");
fn aura_axis(ui: &mut egui::Ui, vs_cool_or_wild: &mut i32, vs_smooth_or_rough: &mut i32) {
	ui.label("Car Aura");
    let img = image::load(Cursor::new(COLOR_IMAGE_BYTES), image::ImageFormat::Png)
        .unwrap()
        .to_rgba8();

    let (width, height) = img.dimensions();
    let color_image = ColorImage::from_rgba_unmultiplied([width as _, height as _], &img);

    let texture: TextureHandle = ui.ctx().load_texture("color_image", color_image, TextureOptions::default());

    let image_response = ui.image(&texture);
    let painter = ui.painter();

    for i in -4..=4 {
        for j in -4..=4 {
            let dot_x = ((i + 4) as f32 / 8.0) * width as f32;
            let dot_y = ((j + 4) as f32 / 8.0) * height as f32;

            if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                let dot_pos = image_response.rect.min + Vec2::new(dot_x, dot_y);
                let distance = (pointer_pos - dot_pos).length();
                if distance < 3.0 && image_response.rect.contains(pointer_pos) {
                    *vs_cool_or_wild = i;
                    *vs_smooth_or_rough = j;
                }
            }
        }
    }

    let current_dot_x = ((*vs_cool_or_wild + 4) as f32 / 8.0) * width as f32;
    let current_dot_y = ((*vs_smooth_or_rough + 4) as f32 / 8.0) * height as f32;

    painter.circle(
        image_response.rect.min + Vec2::new(current_dot_x, current_dot_y),
        5.0, // Radius of the dot
        Color32::WHITE,
        Stroke::new(1.0, Color32::TRANSPARENT),
    );
	ui.end_row();
}

fn set_aero_set(ui: &mut egui::Ui, car: &mut wm::Car, car_items: &[wm::CarItem]) {
    ui.label("Aero Set");
    
    let aero_category = if let Some(selected_car) = wm::Cars::from_u32(car.visual_model()) {
        selected_car.aero_category()
    } else {
        wm::ItemCategory::CatAeroFullset
    };

    let selected = wm::DU_ITEMS
        .iter()
        .find(|&item| item.server_id == car.aero && item.category == aero_category)
        .map_or(String::from("Stock"), |item| item.name.to_string());

    let aero_set: Vec<&wm::DressUpItem> = wm::DU_ITEMS
        .iter()
        .filter(|&item| item.category == aero_category)
        .collect();

    egui::ComboBox::from_id_source("AeroSetComboBox")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut car.aero, 0, "Stock");
            for item in &aero_set {
                if car_items.iter().any(|car_item| {
                    car_item.category == aero_category.into()
                        && Some(car_item.item_id) == item.server_id.to_u32()
                }) {
                    ui.selectable_value(&mut car.aero, item.server_id, item.name);
                }
            }
        });
    ui.end_row();
}

fn set_aero_mirror(ui: &mut egui::Ui, car: &mut wm::Car, car_items: &[wm::CarItem]) {
	ui.label("Aero Mirror");
	let selected = wm::DU_ITEMS
        .iter()
        .find(|&item| item.server_id == car.mirror && item.category == wm::ItemCategory::CatMirror)
        .map_or(String::from("Stock"), |item| item.name.to_string());

    let mirror: Vec<&wm::DressUpItem> = wm::DU_ITEMS
        .iter()
        .filter(|&item| item.category == wm::ItemCategory::CatMirror)
        .collect();

    egui::ComboBox::from_id_source("AeroMirrorComboBox")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut car.mirror, 0, "Stock");
            for item in &mirror {
                if car_items.iter().any(|car_item| {
                    car_item.category == wm::ItemCategory::CatMirror.into()
                        && Some(car_item.item_id) == item.server_id.to_u32()
                }) {
                    ui.selectable_value(&mut car.mirror, item.server_id, item.name);
                }
            }
        });
    ui.end_row();
}

fn set_bonnet(ui: &mut egui::Ui, car: &mut wm::Car, car_items: &[wm::CarItem]) {
    ui.label("Bonnet");
    let selected = wm::DU_ITEMS
        .iter()
        .find(|&item| item.server_id == car.bonnet && item.category == wm::ItemCategory::CatBonnet)
        .map_or(String::from("Stock"), |item| item.name.to_string());

    let bonnet: Vec<&wm::DressUpItem> = wm::DU_ITEMS
        .iter()
        .filter(|&item| item.category == wm::ItemCategory::CatBonnet)
        .collect();

    egui::ComboBox::from_id_source("BonnetComboBox")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut car.bonnet, 0, "Stock");
            for item in &bonnet {
                if car_items.iter().any(|car_item| {
                    car_item.category == wm::ItemCategory::CatBonnet.into()
                        && Some(car_item.item_id) == item.server_id.to_u32()
                }) {
                    ui.selectable_value(&mut car.bonnet, item.server_id, item.name);
                }
            }
        });
    ui.end_row();
}

fn set_trunk(ui: &mut egui::Ui, car: &mut wm::Car, car_items: &[wm::CarItem]) {
	ui.label("Trunk");
	let selected = wm::DU_ITEMS
        .iter()
        .find(|&item| item.server_id == car.trunk && item.category == wm::ItemCategory::CatTrunk)
        .map_or(String::from("Stock"), |item| item.name.to_string());

    let trunk: Vec<&wm::DressUpItem> = wm::DU_ITEMS
        .iter()
        .filter(|&item| item.category == wm::ItemCategory::CatTrunk)
        .collect();

    egui::ComboBox::from_id_source("TrunkComboBox")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut car.trunk, 0, "Stock");
            for item in &trunk {
                if car_items.iter().any(|car_item| {
                    car_item.category == wm::ItemCategory::CatTrunk.into()
                        && Some(car_item.item_id) == item.server_id.to_u32()
                }) {
                    ui.selectable_value(&mut car.trunk, item.server_id, item.name);
                }
            }
        });
    ui.end_row();
}

fn set_neon(ui: &mut egui::Ui, car: &mut wm::Car, car_items: &[wm::CarItem]) {
	ui.label("Neon");
	let selected = wm::DU_ITEMS
        .iter()
        .find(|&item| item.server_id == car.neon && item.category == wm::ItemCategory::CatNeon)
        .map_or(String::from("Stock"), |item| item.name.to_string());

    let neon: Vec<&wm::DressUpItem> = wm::DU_ITEMS
        .iter()
        .filter(|&item| item.category == wm::ItemCategory::CatNeon)
        .collect();

    egui::ComboBox::from_id_source("NeonComboBox")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut car.neon, 0, "Stock");
            for item in &neon {
                if car_items.iter().any(|car_item| {
                    car_item.category == wm::ItemCategory::CatNeon.into()
                        && Some(car_item.item_id) == item.server_id.to_u32()
                }) {
                    ui.selectable_value(&mut car.neon, item.server_id, item.name);
                }
            }
        });
    ui.end_row();
}

fn set_number_plate_frame(ui: &mut egui::Ui, car: &mut wm::Car, car_items: &[wm::CarItem]) {

	#[derive(FromPrimitive, ToPrimitive, EnumIter)]
	enum PlateFrame {
		FrameLogo = 1,
		FrameStripe,
		FrameLuxury,
		FrameIllumination,
	}

	impl ToString for PlateFrame {
		fn to_string(&self) -> String {
			match self {
				PlateFrame::FrameLogo => String::from("Logo Variant"),
				PlateFrame::FrameStripe => String::from("Stripe Variant"),
				PlateFrame::FrameLuxury => String::from("Luxury Variant"),
				PlateFrame::FrameIllumination => String::from("Illumination Variant"),
			}
		}
	}

	#[derive(FromPrimitive, ToPrimitive, EnumIter)]
	enum FrameA {
		YMSPEED = 0,
		MACH,
		RGO,
		ACE,
		R200,
		FLAT,
		BLACKBIRD,
		ZERO,
		GREENAUTO,
		GTCARS,
	}

	impl ToString for FrameA {
		fn to_string(&self) -> String {
			match self {
				FrameA::YMSPEED => String::from("YM SPEED"),
				FrameA::MACH => String::from("MACH"),
				FrameA::RGO => String::from("RGO"),
				FrameA::ACE => String::from("ACE"),
				FrameA::R200 => String::from("R200"),
				FrameA::FLAT => String::from("FLAT"),
				FrameA::BLACKBIRD => String::from("BLACK BIRD"),
				FrameA::ZERO => String::from("ZERO"),
				FrameA::GREENAUTO => String::from("GREEN AUTO"),
				FrameA::GTCARS => String::from("GT CARS"),
			}
		}
	}

	#[derive(FromPrimitive, ToPrimitive, EnumIter)]
	enum FrameB {
		Red = 0,
		Orange,
		Yellow,
		Green,
		Purple,
		Teal,
		Blue,
		White,
	}

	impl ToString for FrameB {
		fn to_string(&self) -> String {
			match self {
				FrameB::Red => String::from("Red"),
				FrameB::Orange => String::from("Orange"),
				FrameB::Yellow => String::from("Yellow"),
				FrameB::Green => String::from("Green"),
				FrameB::Purple => String::from("Purple"),
				FrameB::Teal => String::from("Teal"),
				FrameB::Blue => String::from("Blue"),
				FrameB::White => String::from("White"),
			}
		}
	}

	#[derive(FromPrimitive, ToPrimitive, EnumIter)]
	enum FrameC {
		White = 0,
		Black,
		Gold,
	}

	impl ToString for FrameC {
		fn to_string(&self) -> String {
			match self {
				FrameC::White => String::from("White"),
				FrameC::Black => String::from("Black"),
				FrameC::Gold => String::from("Gold"),
			}
		}
	}

	#[derive(FromPrimitive, ToPrimitive, EnumIter)]
	enum FrameD {
		Green = 0,
		Blue,
		Lightpurple,
		Red,
		Yellow,
		Purple,
	}

	impl ToString for FrameD {
		fn to_string(&self) -> String {
			match self {
				FrameD::Green => String::from("Green"),
				FrameD::Blue => String::from("Blue"),
				FrameD::Lightpurple => String::from("Light Purple"),
				FrameD::Red => String::from("Red"),
				FrameD::Yellow => String::from("Yellow"),
				FrameD::Purple => String::from("Purple"),
			}
		}
	}
	
	ui.label("Number Plate Frame");
	let selected = match car.plate {
        plate_value => match PlateFrame::from_u32(plate_value) {
            Some(plate) => plate.to_string(),
            None => String::from("Stock"),
		}
    };

	ui.horizontal(|ui| {
		egui::ComboBox::from_id_source("NumberPlateFrameComboBox")
			.selected_text(selected)
			.show_ui(ui, |ui| {
				ui.selectable_value(&mut car.plate, 0, "Stock");
				for plate in PlateFrame::iter() {
					if car_items.iter().any(|item| {
						item.category == wm::ItemCategory::CatNumberPlate.into()
							&& Some(item.item_id) == plate.to_u32()
					}) {
						ui.selectable_value(
							&mut car.plate,
							plate.to_u32().unwrap(),
							plate.to_string(),
						);
					}
				}
			});

		if car.plate == 1 {
			ui.label("Car Frame Color");
			let selected_color = match FrameA::from_u32(car.plate_color) {
				Some(plate_color) => plate_color.to_string(),
				None => String::from("Select Plate Color"),
			};

			egui::ComboBox::from_id_source("FrameAComboBox")
				.selected_text(selected_color)
				.show_ui(ui, |ui| {
					for plate_color in FrameA::iter() {
						ui.selectable_value(
							&mut car.plate_color,
							plate_color.to_u32().unwrap(),
							plate_color.to_string(),
						);
					}
				});
		} else if car.plate == 2 {
			ui.label("Car Frame Color");
			let selected_color = match FrameB::from_u32(car.plate_color) {
				Some(plate_color) => plate_color.to_string(),
				None => String::from("Select Plate Color"),
			};

			egui::ComboBox::from_id_source("FrameBComboBox")
				.selected_text(selected_color)
				.show_ui(ui, |ui| {
					for plate_color in FrameB::iter() {
						ui.selectable_value(
							&mut car.plate_color,
							plate_color.to_u32().unwrap(),
							plate_color.to_string(),
						);
					}
				});
		} else if car.plate == 3 {
			ui.label("Car Frame Color");
			let selected_color = match FrameC::from_u32(car.plate_color) {
				Some(plate_color) => plate_color.to_string(),
				None => String::from("Select Plate Color"),
			};

			egui::ComboBox::from_id_source("FrameCComboBox")
				.selected_text(selected_color)
				.show_ui(ui, |ui| {
					for plate_color in FrameC::iter() {
						ui.selectable_value(
							&mut car.plate_color,
							plate_color.to_u32().unwrap(),
							plate_color.to_string(),
						);
					}
				});
		} else if car.plate == 4 {
			ui.label("Car Frame Color");
			let selected_color = match FrameD::from_u32(car.plate_color) {
				Some(plate_color) => plate_color.to_string(),
				None => String::from("Select Plate Color"),
			};

			egui::ComboBox::from_id_source("FrameDComboBox")
				.selected_text(selected_color)
				.show_ui(ui, |ui| {
					for plate_color in FrameD::iter() {
						ui.selectable_value(
							&mut car.plate_color,
							plate_color.to_u32().unwrap(),
							plate_color.to_string(),
						);
					}
				});
		}
	});	
	ui.end_row();
}

fn set_wing(ui: &mut egui::Ui, car: &mut wm::Car, car_items: &[wm::CarItem], custom:&mut bool) {
    #[derive(FromPrimitive, ToPrimitive, EnumIter)]
    enum Wing {
        GtA = 1,
        ModelA,
        GtB,
        ModelB,
        GtC,
        GtD,
        ModelC,
        GtE,
		Wingless = 127,
    }

    impl ToString for Wing {
        fn to_string(&self) -> String {
            match self {
                Wing::GtA => String::from("GT Wing A (Straight)"),
                Wing::ModelA => String::from("Car Model Wing A"),
                Wing::GtB => String::from("GT Wing B (3D)"),
                Wing::ModelB => String::from("Car Model Wing B"),
                Wing::GtC => String::from("GT Wing C (3D2)"),
                Wing::GtD => String::from("GT Wing D (Twin)"),
                Wing::ModelC => String::from("Car Model Wing C"),
                Wing::GtE => String::from("GT Wing E"),
				Wing::Wingless => String::from("Wingless"),
            }
        }
    }

    #[derive(FromPrimitive, ToPrimitive, EnumIter)]
    enum GtPillar {
        TallWide = 1,
        TallNarrow,
        NormalWide,
        NormalNarrow,
        LowWide,
        LowNarrow,
    }

    impl ToString for GtPillar {
        fn to_string(&self) -> String {
            match self {
                GtPillar::TallWide => String::from("Tall Wide"),
                GtPillar::TallNarrow => String::from("Tall Narrow"),
                GtPillar::NormalWide => String::from("Normal Wide"),
                GtPillar::NormalNarrow => String::from("Normal Narrow"),
                GtPillar::LowWide => String::from("Low Wide"),
                GtPillar::LowNarrow => String::from("Low Narrow"),
            }
        }
    }

    #[derive(FromPrimitive, ToPrimitive, EnumIter)]
    enum GtPillarMat {
        BlackPillar = 0,
        WhitePillar,
    }

    impl ToString for GtPillarMat {
        fn to_string(&self) -> String {
            match self {
                GtPillarMat::BlackPillar => String::from("Black Pillar"),
                GtPillarMat::WhitePillar => String::from("White Pillar"),
            }
        }
    }

    #[derive(FromPrimitive, ToPrimitive, EnumIter)]
    enum GtMainWing {
        Straight = 1,
        Curve,
        ThreeD,
        StraightBig,
        ThreeDCurve,
        Twin,
        CurveBig,
    }

    impl ToString for GtMainWing {
        fn to_string(&self) -> String {
            match self {
                GtMainWing::Straight => String::from("Straight"),
                GtMainWing::Curve => String::from("Curve"),
                GtMainWing::ThreeD => String::from("3D"),
                GtMainWing::StraightBig => String::from("Big Straight"),
                GtMainWing::ThreeDCurve => String::from("3D Curve"),
                GtMainWing::Twin => String::from("Twin"),
                GtMainWing::CurveBig => String::from("Big Curve"),
            }
        }
    }

    #[derive(FromPrimitive, ToPrimitive, EnumIter)]
    enum GtMainWingColor {
        Red = 0,
        Orange,
        Yellow,
        Green,
        Purple,
        Teal,
        Blue,
        Black,
        Silver,
        White,
    }

    impl ToString for GtMainWingColor {
        fn to_string(&self) -> String {
            match self {
                GtMainWingColor::Red => String::from("Red"),
                GtMainWingColor::Orange => String::from("Orange"),
                GtMainWingColor::Yellow => String::from("Yellow"),
                GtMainWingColor::Green => String::from("Green"),
                GtMainWingColor::Purple => String::from("Purple"),
                GtMainWingColor::Teal => String::from("Teal"),
                GtMainWingColor::Blue => String::from("Blue"),
                GtMainWingColor::Black => String::from("Black"),
                GtMainWingColor::Silver => String::from("Silver"),
                GtMainWingColor::White => String::from("White"),
            }
        }
    }

    #[derive(FromPrimitive, ToPrimitive, EnumIter)]
    enum GtWingTip {
        Variant1 = 1,
        Variant2,
        Variant3,
        Variant4,
    }

    impl ToString for GtWingTip {
        fn to_string(&self) -> String {
            match self {
                GtWingTip::Variant1 => String::from("Variant 1"),
                GtWingTip::Variant2 => String::from("Variant 2"),
                GtWingTip::Variant3 => String::from("Variant 3"),
                GtWingTip::Variant4 => String::from("Variant 4"),
            }
        }
    }

    #[derive(FromPrimitive, ToPrimitive, EnumIter)]
    enum GtWingMaterial {
        Carbon = 0,
        Gloss,
    }

    impl ToString for GtWingMaterial {
        fn to_string(&self) -> String {
            match self {
                GtWingMaterial::Carbon => String::from("Carbon"),
                GtWingMaterial::Gloss => String::from("Gloss"),
            }
        }
    }

    ui.horizontal(|ui| {
		ui.label("Wing");
		if car_items.iter().any(|item| {
			item.category == wm::ItemCategory::CatGtWing.into()
				&& Some(item.item_id) == Some(1)
		}) {
			ui.checkbox(custom, "Custom GT Wng");
		}
	});

	let selected = match car.wing {
		wing_value => match Wing::from_u32(wing_value) {
			Some(wing) => wing.to_string(),
			None => String::from("Stock"),
		}
	};

	egui::ComboBox::from_id_source("WingComboBox")
		.selected_text(selected)
		.show_ui(ui, |ui| {
			ui.selectable_value(&mut car.wing, 0, "Stock");
			for wing in Wing::iter() {
				if car_items.iter().any(|item| {
					item.category == wm::ItemCategory::CatWing.into()
						&& Some(item.item_id) == wing.to_u32()
				}) {
					ui.selectable_value(
						&mut car.wing,
						wing.to_u32().unwrap(),
						wing.to_string(),
					);
				}
			}
			ui.selectable_value(&mut car.wing, 127, "Wingless");
		});
		
	ui.end_row();

    if car.wing == 127 && *custom {
        ui.label("GT Pillar");
		let selected_pillar = match &car.gt_wing {
			Some(gt_wing) => match GtPillar::from_u32(gt_wing.pillar) {
				Some(pillar) => pillar.to_string(),
				None => String::from("Select GT Wing Pillar"),
			},
			None => String::from("Select GT Wing Pillar"),
		};

		egui::ComboBox::from_id_source("GtPillarComboBox")
			.selected_text(selected_pillar)
			.show_ui(ui, |ui| {
				for pillar in GtPillar::iter() {
					ui.selectable_value(
						&mut car.gt_wing.as_mut().unwrap().pillar,
						pillar.to_u32().unwrap_or(3),
						pillar.to_string(),
					);
				}
			});
		ui.end_row();

		ui.label("GT Pillar Material");
		let selected_pillar_mat = match &car.gt_wing {
			Some(gt_wing) => match GtPillarMat::from_u32(gt_wing.pillar_material) {
				Some(pillar_mat) => pillar_mat.to_string(),
				None => String::from("Select GT Wing Pillar Material"),
			},
			None => String::from("Select GT Wing Pillar Material"),
		};

		egui::ComboBox::from_id_source("GtPillarMatComboBox")
			.selected_text(selected_pillar_mat)
			.show_ui(ui, |ui| {
				for pillar_mat in GtPillarMat::iter() {
					ui.selectable_value(
						&mut car.gt_wing.as_mut().unwrap().pillar_material,
						pillar_mat.to_u32().unwrap_or(0),
						pillar_mat.to_string(),
					);
				}
			});
		ui.end_row();

		ui.label("GT Main Wing");
		let selected_main_wing = match &car.gt_wing {
			Some(gt_wing) => match GtMainWing::from_u32(gt_wing.main_wing) {
				Some(main_wing) => main_wing.to_string(),
				None => String::from("Select GT Main Wing"),
			},
			None => String::from("Select GT Main Wing"),
		};

		egui::ComboBox::from_id_source("GtMainWingComboBox")
			.selected_text(selected_main_wing)
			.show_ui(ui, |ui| {
				for main_wing in GtMainWing::iter() {
					ui.selectable_value(
						&mut car.gt_wing.as_mut().unwrap().main_wing,
						main_wing.to_u32().unwrap_or(1),
						main_wing.to_string(),
					);
				}
			});
		ui.end_row();

		ui.label("GT Main Wing Color");
		let selected_main_wing_color = match &car.gt_wing {
			Some(gt_wing) => match GtMainWingColor::from_u32(gt_wing.main_wing_color) {
				Some(main_wing_color) => main_wing_color.to_string(),
				None => String::from("Select GT Main Wing Color"),
			},
			None => String::from("Select GT Main Wing Color"),
		};

		egui::ComboBox::from_id_source("GtMainWingColorComboBox")
			.selected_text(selected_main_wing_color)
			.show_ui(ui, |ui| {
				for main_wing_color in GtMainWingColor::iter() {
					ui.selectable_value(
						&mut car.gt_wing.as_mut().unwrap().main_wing_color,
						main_wing_color.to_u32().unwrap_or(7),
						main_wing_color.to_string(),
					);
				}
			});
		ui.end_row();

		ui.label("GT Wing Tip");
		let selected_wing_tip = match &car.gt_wing {
			Some(gt_wing) => match GtWingTip::from_u32(gt_wing.wing_tip) {
				Some(wing_tip) => wing_tip.to_string(),
				None => String::from("Select GT Wing Tip"),
			},
			None => String::from("Select GT Wing Tip"),
		};

		egui::ComboBox::from_id_source("GtWingTipComboBox")
			.selected_text(selected_wing_tip)
			.show_ui(ui, |ui| {
				for wing_tip in GtWingTip::iter() {
					ui.selectable_value(
						&mut car.gt_wing.as_mut().unwrap().wing_tip,
						wing_tip.to_u32().unwrap_or(1),
						wing_tip.to_string(),
					);
				}
			});
		ui.end_row();

		ui.label("GT Wing Material");
		let selected_wing_material = match &car.gt_wing {
			Some(gt_wing) => match GtWingMaterial::from_u32(gt_wing.material) {
				Some(wing_material) => wing_material.to_string(),
				None => String::from("Select GT Wing Material"),
			},
			None => String::from("Select GT Wing Material"),
		};

		egui::ComboBox::from_id_source("GtWingMaterialComboBox")
			.selected_text(selected_wing_material)
			.show_ui(ui, |ui| {
				for wing_material in GtWingMaterial::iter() {
					ui.selectable_value(
						&mut car.gt_wing.as_mut().unwrap().material,
						wing_material.to_u32().unwrap_or(0),
						wing_material.to_string(),
					);
				}
			});
        ui.end_row();
    } else {
        car.gt_wing = Some(wm::GtWing {
            pillar: 0,
            pillar_material: 0,
            main_wing: 0,
            main_wing_color: 0,
            wing_tip: 0,
            material: 0,
        });
    }
}

fn set_wheel(ui: &mut egui::Ui, car: &mut wm::Car, car_items: &[wm::CarItem], force: &mut bool) {
    ui.horizontal(|ui| {
		ui.label("Wheel");
		ui.checkbox(force, "Force Equip");
	});

    let selected = match car.wheel {
        wheel_value => match wm::Wheel::from_u32(wheel_value) {
            Some(wheel) => wheel.to_string(),
            None => String::from("Stock"),
        },
    };

	ui.horizontal(|ui| {
		if *force {
			egui::ComboBox::from_id_source("ForceWheelComboBox")
				.selected_text(selected)
				.show_ui(ui, |ui| {
					ui.selectable_value(&mut car.wheel, 0, "Stock");
					for wheel in wm::Wheel::iter() {
						ui.selectable_value(
							&mut car.wheel,
							wheel.to_u32().unwrap(),
							wheel.to_string()
						);
					}
				});
	
				if let Some(wheel) = wm::Wheel::from_u32(car.wheel) {
					if car.wheel != 0 {
						let color_count = wheel.get_color_count();
						if color_count > 0 {
							ui.label("Wheel Color");
					
							let selected_color_text = format!("Color {}", car.wheel_color + 1);
					
							egui::ComboBox::from_id_source("WheelColorComboBox")
								.selected_text(selected_color_text)
								.show_ui(ui, |ui| {
									for i in 0..color_count {
										let color_text = format!("Color {}", i + 1);
										ui.selectable_value(
											&mut car.wheel_color,
											i as u32,
											color_text,
										);
									}
								});
						}
					}
				}
		} else {
			egui::ComboBox::from_id_source("WheelComboBox")
				.selected_text(selected)
				.show_ui(ui, |ui| {
					ui.selectable_value(&mut car.wheel, 0, "Stock");
					for wheel in wm::Wheel::iter() {
						if car_items.iter().any(|item| {
							item.category == wm::ItemCategory::CatWheel.into()
								&& Some(item.item_id) == wheel.to_u32()
						}) {
							ui.selectable_value(
								&mut car.wheel,
								wheel.to_u32().unwrap(),
								wheel.to_string()
							);
						}
					}
				});
	
				if let Some(wheel) = wm::Wheel::from_u32(car.wheel) {
					if car.wheel != 0 {
						let color_count = wheel.get_color_count();
						if color_count > 0 {
							ui.label("Wheel Color");
					
							let selected_color_text = format!("Color {}", car.wheel_color + 1);
					
							egui::ComboBox::from_id_source("WheelColorComboBox")
								.selected_text(selected_color_text)
								.show_ui(ui, |ui| {
									for i in 0..color_count {
										let color_text = format!("Color {}", i + 1);
										ui.selectable_value(
											&mut car.wheel_color,
											i as u32,
											color_text,
										);
									}
								});
						}
					}
				}
			}
	});
	ui.end_row();
}

async fn save_car(
	server: &Url,
	car: &wm::Car,
	car_settings: &wm::CarSetting,
	play_count: u32,
	odometer: u32,
	vs_star_count: u32,
	vs_cool_or_wild: i32,
	vs_smooth_or_rough: i32,
	vs_play_count: u32,
	vs_gold_medal: u32,
	vs_silver_medal: u32,
	vs_bronze_medal: u32,
	vs_plain_medal: u32,
) -> Result<wm::SaveGameResultResponse> {
	let req = wm::SaveGameResultRequest {
		car_id: car.car_id(),
		game_mode: wm::GameMode::ModeVsBattle.into(),
		played_at: car.last_played_at(),
		play_count,
		car: Some(car.clone()),
		setting: Some(car_settings.clone()),
		odometer: Some(odometer),
		earned_custom_color: Some(false),
		retired: false,
		vs_result: Some(wm::save_game_result_request::VersusBattleResult {
			result: 0,
			survived: true,
			// Bayshore appears to not care
			num_of_players: 0,
			area: 0,
			is_morning: false,
			vs_play_count,
			vs_star_count: Some(vs_star_count),
			vs_triple_star_medals: Some(vs_gold_medal),
			vs_double_star_medals: Some(vs_silver_medal),
			vs_single_star_medals: Some(vs_bronze_medal),
			vs_plain_medals: Some(vs_plain_medal),
			vs_cool_or_wild: Some(vs_cool_or_wild),
			vs_smooth_or_rough: Some(vs_smooth_or_rough),
			..Default::default()
		}),
		..Default::default()
	};

	wm::send_request(req, server, "method/save_game_result").await
}

async fn update_car(
	server: &Url,
	car: &wm::Car,
	car_settings: &wm::CarSetting
) -> Result<wm::UpdateCarResponse> {
	let req = wm::UpdateCarRequest {
		car: Some(car.clone()),
		car_id: car.car_id(),
		setting: Some(car_settings.clone()),

		..Default::default()
	};

	wm::send_request(req, server, "method/update_car").await
}