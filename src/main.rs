use std::{fs::File, io::Read, path::Path};

use eframe::{
    egui::{self, CentralPanel, ImageButton, RichText, TextStyle, Window},
    epaint::Color32,
    run_native, App, CreationContext, NativeOptions,
};
use egui_extras::RetainedImage;
use lazy_static::lazy_static;
use reqwest::header::CONTENT_TYPE;
use rfd::FileDialog;

lazy_static! {
    static ref ERRWIN: bool = true;
}

fn main() {
    let np = NativeOptions::default();
    run_native("Rview", np, Box::new(|cc| Box::new(Rview::new(cc)))).unwrap();
}

#[derive(Default)]
struct Rview {
    image_seted: bool,
    image: String,
    image_retained: Option<RetainedImage>,
    errwin: bool,
    errmsg: String,
}

impl Rview {
    fn new(_cc: &CreationContext<'_>) -> Self {
        Self {
            image_seted: true,
            ..Default::default()
        }
    }
}

impl App for Rview {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            Window::new(monospace_text("Oops!"))
                .open(&mut self.errwin)
                .show(ctx, |ui| {
                    ui.heading(
                        monospace_text(format!("Error due To: {}", self.errmsg))
                            .color(Color32::RED),
                    );
                });
            if let Some(image_retained) = &self.image_retained {
                if ui
                    .add(ImageButton::new(
                        image_retained.texture_id(ctx),
                        frame.info().window_info.size,
                    ))
                    .secondary_clicked()
                {
                    self.image_seted = !self.image_seted;
                }
            }

            Window::new(monospace_text("Rview"))
                .title_bar(false)
                .open(&mut self.image_seted.clone())
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(monospace_text("Set Image Url Or File Path!"));

                        ui.label(monospace_text("Enter File Path or Url:"));
                        ui.add_space(5.);
                        ui.text_edit_singleline(&mut self.image);
                        ui.add_space(5.);
                        ui.vertical_centered_justified(|ui| {
                            if ui.button(monospace_text("Select From File!")).clicked() {
                                if let Some(path) = FileDialog::new()
                                    .add_filter(
                                        "image",
                                        &[
                                            "gif", "jpeg", "ico", "png", "pnm", "tga", "tiff",
                                            "jpg", "svg", "webp", "bmp", "hdr", "dxt", "dds",
                                            "farbfeld", "openexr",
                                        ],
                                    )
                                    .pick_file()
                                {
                                    let path = format!("{:?}", path);
                                    self.image = path[1..path.len() - 1].to_string()
                                }
                            }
                            ui.add_space(5.);
                            if ui.button(monospace_text("Open Image")).clicked() {
                                let mut bytes = vec![];
                                if url::Url::parse(&self.image).is_ok() {
                                    match image_req(&self.image) {
                                        Ok(bytess) => bytes = bytess,
                                        Err(e) => {
                                            self.errwin = true;
                                            self.errmsg = e.to_string();
                                        }
                                    }
                                } else if Path::new(&self.image).exists() {
                                    match File::open(&self.image) {
                                        Ok(mut file) => {
                                            if let Err(e) = file.read_to_end(&mut bytes) {
                                                self.errwin = true;
                                                self.errmsg = e.to_string()
                                            }
                                        }
                                        Err(e) => {
                                            self.errwin = true;
                                            self.errmsg = e.to_string()
                                        }
                                    }
                                } else {
                                    self.errwin = true;
                                    self.errmsg = format!(
                                        "Your Image `{}`, is invalid Url Or File Path!",
                                        self.image
                                    );
                                }

                                if !bytes.is_empty() {
                                    let utf8_bytes = String::from_utf8_lossy(&bytes);
                                    if utf8_bytes.contains("<?xml") || utf8_bytes.contains("<svg") {
                                        match RetainedImage::from_svg_bytes(&self.image, &bytes) {
                                            Ok(image_retained) => {
                                                self.image_retained = Some(image_retained);
                                            }
                                            Err(e) => {
                                                self.errwin = true;
                                                self.errmsg = e.to_string();
                                            }
                                        }
                                    } else {
                                        match RetainedImage::from_image_bytes(&self.image, &bytes) {
                                            Ok(image_retained) => {
                                                self.image_retained = Some(image_retained);
                                            }
                                            Err(e) => {
                                                self.errwin = true;
                                                self.errmsg = e.to_string();
                                            }
                                        }
                                    }
                                    self.image_seted = false;
                                } else {
                                    self.errwin = true;
                                    self.errmsg = format!(
                                        "Reading bytes From Your Image `{}` is Error, Maybe image format is not supported?",
                                        self.image
                                    );
                                }
                            }
                        })
                    })
                });
        });
    }
}

fn image_req(url: &str) -> Result<Vec<u8>, String> {
    match reqwest::blocking::get(url) {
        Ok(resp) => match resp.headers().get(CONTENT_TYPE) {
            Some(content_type) => match content_type.to_str() {
                Ok(ctstr) => {
                    if ctstr.starts_with("image") {
                        match resp.bytes() {
                            Ok(bytes) => Ok(bytes.to_vec()),
                            Err(e) => Err(e.to_string()),
                        }
                    } else {
                        Err("Url is Not Image's Url??".to_string())
                    }
                }
                Err(e) => Err(e.to_string()),
            },
            None => Err("Url Is Not Image's Url??".to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

fn monospace_text(text: impl Into<String>) -> RichText {
    RichText::new(text).text_style(TextStyle::Monospace)
}
