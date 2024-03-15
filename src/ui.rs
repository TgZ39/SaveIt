use crate::database::{establish_connection, get_all_sources, insert_source, Source};
use arboard::Clipboard;
use egui::FontFamily::Proportional;
use egui::TextStyle::*;
use egui::{FontId, Grid, Ui};
use egui_extras::DatePickerButton;
use futures::executor;
use std::fmt::{Display, Formatter};
use chrono::{Local, NaiveDate};

pub fn open_gui() -> Result<(), eframe::Error> {
    // set up logging
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 350.0])
            .with_min_inner_size([500.0, 350.0]),
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
    sources_cache: Vec<Source>, // cache needed because every time the user interacted (e.g. mouse movement) with the ui, a new DB request would be made. (~60/s)
}

impl Application {
    fn new(ctx: &egui::Context) -> Self {
        // make font bigger
        configure_fonts(ctx);

        Self {
            input_url: String::new(),
            input_author: String::new(),
            input_date: NaiveDate::from(Local::now().naive_local()), // Current date
            curr_page: AppPage::Start,
            sources_cache: vec![],
        }
    }

    // get input source from user
    fn get_source(&self) -> Source {
        Source {
            url: self.input_url.clone(),
            author: self.input_author.clone(),
            date: self.input_date,
        }
    }

    // save input source to DB
    pub fn handle_source_save(&self) {
        // run async fn in sync code ¯\_(ツ)_/¯
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

    // clears text fields and reset date to now
    fn clear_input(&mut self) {
        self.input_url.clear();
        self.input_author.clear();
        self.input_date = NaiveDate::from(Local::now().naive_local());
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
    // runs every frame
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Page selection
            ui.horizontal(|ui| {
                // Start page
                ui.selectable_value(
                    &mut self.curr_page,
                    AppPage::Start,
                    AppPage::Start.to_string(),
                );

                // List page
                let list_page = ui.selectable_value(
                    &mut self.curr_page,
                    AppPage::List,
                    AppPage::List.to_string(),
                );

                if list_page.clicked() {
                    // update source cache
                    executor::block_on(async {
                        let mut conn = establish_connection()
                            .await
                            .expect("Error connecting to database.");

                        self.sources_cache = get_all_sources(&mut conn)
                            .await
                            .expect("Error loading sources.");
                    });
                }

                // Settings page
                ui.selectable_value(
                    &mut self.curr_page,
                    AppPage::Settings,
                    AppPage::Settings.to_string(),
                );
            });

            ui.separator();

            // render selected page
            match self.curr_page {
                AppPage::Start => render_start_page(self, ui),
                AppPage::List => render_list_page(ui, &self.sources_cache),
                AppPage::Settings => {}
            }
        });
    }
}

fn render_start_page(app: &mut Application, ui: &mut Ui) {
    Grid::new("SourceInput").num_columns(2).show(ui, |ui| {
        let url_label = ui.label("URL: ");

        // input URL
        ui.text_edit_singleline(&mut app.input_url)
            .labelled_by(url_label.id);
        ui.end_row();

        // input author
        let author_label = ui.label("Author: ");
        ui.text_edit_singleline(&mut app.input_author)
            .labelled_by(author_label.id);
        ui.end_row();

        // input date
        let date_label = ui.label("Date: ");
        ui.add(DatePickerButton::new(&mut app.input_date))
            .labelled_by(date_label.id);
        ui.end_row();
    });

    ui.add_space(10.0);

    ui.horizontal(|ui| {
        // save input source to DB
        if ui.button("Save").clicked() {
            app.handle_source_save();
        }

        // clear input
        if ui.button("Clear").clicked() {
            app.clear_input();
        }
    });
}

fn configure_fonts(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.text_styles = [
        (Heading, FontId::default()),
        (Body, FontId::new(15.0, Proportional)), // TODO making fontsize above 15 breaks date selection popup
        (Monospace, FontId::default()),
        (Button, FontId::default()),
        (Small, FontId::default()),
    ]
    .into();

    ctx.set_style(style);
}

fn render_list_page(ui: &mut Ui, sources: &Vec<Source>) {
    if ui.button("Copy all").clicked() {
        set_all_clipboard(sources)
    }

    ui.add_space(10.0);

    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .drag_to_scroll(true)
        .show(ui, |ui| {
            // fancy table
            Grid::new("SourceList")
                .striped(true)
                .num_columns(2)
                .show(ui, |ui| {
                    for source in sources {
                        // copy source to clipboard
                        if ui.button("Copy").clicked() {
                            set_clipboard(source)
                        }
                        // show URL
                        ui.label(source.url.to_string());
                        ui.end_row()
                    }
                });
        });
}

fn set_clipboard(source: &Source) {
    let mut clipboard = Clipboard::new().unwrap();

    let text = source.format();

    clipboard.set_text(text).unwrap();
}

fn set_all_clipboard(sources: &Vec<Source>) {
    let mut clipboard = Clipboard::new().unwrap();

    let mut text = "".to_string();

    for source in sources {
        text.push_str(source.format().as_str());
        text.push('\n');
    }

    clipboard.set_text(text).unwrap();
}
