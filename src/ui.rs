use crate::database::{establish_connection, insert_source, Source};
use chrono::{Local, NaiveDate};
use egui::FontFamily::Proportional;
use egui::TextStyle::*;
use egui::{FontId, Ui};
use egui_extras::DatePickerButton;
use futures::executor;
use std::fmt::{Display, Formatter};

pub fn open_gui() -> Result<(), eframe::Error> {
    // set up logging
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([500.0, 350.0]),
        ..Default::default()
    };

    // open GUI
    eframe::run_native(
        format!("SaveIt v{}", env!("CARGO_PKG_VERSION")).as_str(),
        options,
        Box::new(|cc| Box::new(Application::new(&cc.egui_ctx))),
    )
}

pub struct Application {
    pub input_url: String,
    pub input_author: String,
    pub input_date: NaiveDate,
    curr_page: AppPage,
}

impl Application {
    fn new(ctx: &egui::Context) -> Self {
        configure_fonts(ctx);

        Self {
            input_url: String::new(),
            input_author: String::new(),
            input_date: NaiveDate::from(Local::now().naive_local()), // Current date
            curr_page: AppPage::Start,
        }
    }

    fn get_input_source(&self) -> Source {
        Source {
            url: self.input_url.clone(),
            author: self.input_author.clone(),
            date: self.input_date,
        }
    }

    pub fn handle_source_save(&self) {
        executor::block_on(async {
            let source = self.get_source();

            let mut conn = establish_connection()
                .await
                .expect("Error connecting to database.");
            insert_source(&mut conn, &source)
                .await
                .expect("Error inserting source in database.");
        });
    }
}

#[derive(PartialOrd, PartialEq)]
enum AppPage {
    Start,
    List,
    Settings,
}

impl Display for AppPage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AppPage::Start => {
                write!(f, "Start")
            }
            AppPage::List => {
                write!(f, "List")
            }
            AppPage::Settings => {
                write!(f, "Settings")
            }
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Page selection
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.curr_page,
                    AppPage::Start,
                    AppPage::Start.to_string(),
                );
                ui.selectable_value(
                    &mut self.curr_page,
                    AppPage::List,
                    AppPage::List.to_string(),
                );
                ui.selectable_value(
                    &mut self.curr_page,
                    AppPage::Settings,
                    AppPage::Settings.to_string(),
                );
            });

            ui.separator();
            match self.curr_page {
                AppPage::Start => render_start_page(self, ui),
                AppPage::List => {}
                AppPage::Settings => {}
            }
        });
    }
}

fn render_start_page(app: &mut Application, ui: &mut Ui) {
    egui::Grid::new("SourceInput")
        .num_columns(2)
        .show(ui, |ui| {
            let url_label = ui.label("URL: ");
            ui.text_edit_singleline(&mut app.input_url)
                .labelled_by(url_label.id);
            ui.end_row();

            let author_label = ui.label("Author: ");
            ui.text_edit_singleline(&mut app.input_author)
                .labelled_by(author_label.id);
            ui.end_row();

            let date_label = ui.label("Date: ");
            ui.add(DatePickerButton::new(&mut app.input_date))
                .labelled_by(date_label.id);
            ui.end_row();
        });

    let save_button = ui.button("Save");

    if save_button.clicked() {
        app.handle_source_save()
    }
}

fn configure_fonts(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.text_styles = [
        (Heading, FontId::default()),
        (Body, FontId::new(15.0, Proportional)),
        (Monospace, FontId::default()),
        (Button, FontId::default()),
        (Small, FontId::default()),
    ]
    .into();

    ctx.set_style(style);
}
