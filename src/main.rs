use std::{fs::File, io::Read, path::Path};

use eframe::{
    egui::{self, CentralPanel, ImageButton, RichText, TextStyle, Window},
    run_native, App, CreationContext, NativeOptions,
};
use egui_extras::RetainedImage;
use reqwest::header::CONTENT_TYPE;
use rfd::FileDialog;

fn main() {
    let np = NativeOptions::default();
    run_native("Rview", np, Box::new(|cc| Box::new(Rview::new(cc)))).unwrap();
}

#[derive(Default)]
struct Rview {
    image_seted: bool,
    image: String,
    image_bytes: Vec<u8>,
    image_retained: Option<RetainedImage>,
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
                                    bytes = image_req(&self.image).unwrap();
                                } else if Path::new(&self.image).exists() {
                                    let mut file = File::open(&self.image).unwrap();
                                    file.read_to_end(&mut bytes).unwrap();
                                } else {
                                    todo!()
                                }

                                if !bytes.is_empty() {
                                    self.image_bytes = bytes.clone();
                                    let utf8_bytes = String::from_utf8_lossy(&bytes);
                                    if utf8_bytes.contains("<?xml") || utf8_bytes.contains("<svg") {
                                        self.image_retained = Some(
                                            RetainedImage::from_svg_bytes(&self.image, &bytes)
                                                .unwrap(),
                                        );
                                    } else {
                                        self.image_retained = Some(
                                            RetainedImage::from_image_bytes(&self.image, &bytes)
                                                .unwrap(),
                                        );
                                    }
                                    self.image_seted = false;
                                }
                            }
                        })
                    })
                });
        });
    }
}

fn image_req(url: &str) -> Result<Vec<u8>, ()> {
    let resp = reqwest::blocking::get(url).unwrap();

    if let Some(content_type) = resp.headers().get(CONTENT_TYPE) {
        if content_type.to_str().unwrap().starts_with("image") {
            let bytes = resp.bytes().unwrap().to_vec();
            Ok(bytes)
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

fn monospace_text(text: impl Into<String>) -> RichText {
    RichText::new(text).text_style(TextStyle::Monospace)
}
