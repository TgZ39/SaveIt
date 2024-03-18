use std::fmt::{Display, Formatter};
use std::sync::{Arc, RwLock};

use crate::config::{Config, FormatStandard};
use arboard::Clipboard;
use chrono::{Local, NaiveDate};
use egui::scroll_area::ScrollBarVisibility;
use egui::text::LayoutJob;
use egui::FontFamily::Proportional;
use egui::TextStyle::*;
use egui::{text, CentralPanel, ComboBox, Context, FontId, Grid, TextFormat, Ui, TextEdit};
use egui_extras::DatePickerButton;

use crate::database::{delete_source, get_all_sources, insert_source, update_source, Source};

pub struct Application {
    pub input_title: String,
    pub input_url: String,
    pub input_author: String,
    pub input_published_date: NaiveDate,
    input_published_enabled: bool,
    pub input_viewed_date: NaiveDate,
    curr_page: AppPage,
    sources_cache: Arc<RwLock<Vec<Source>>>,
    // cache needed because every time the user interacted (e.g. mouse movement) with the ui, a new DB request would be made. (30-60/s)
    edit_windows_open: bool,
    // using cell for more convenient editing of this value (btw fuck the borrow checker)
    edit_source: Source,
    source_format: FormatStandard,
}

pub fn open_gui() -> Result<(), eframe::Error> {
    // set up logging
    env_logger::init();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([500.0, 350.0])
        .with_min_inner_size([500.0, 350.0]);

    // load icon
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"));

    if let Ok(icon_data) = icon {
        viewport = viewport.with_icon(icon_data);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    // open GUI
    eframe::run_native(
        format!("SaveIt v{}", env!("CARGO_PKG_VERSION")).as_str(),
        options,
        Box::new(|cc| Box::new(Application::new(&cc.egui_ctx))),
    )
}

macro_rules! text_label_wrapped {
    ($text:expr, $ui:expr) => {
        let mut job = LayoutJob::single_section($text.to_string(), TextFormat::default());

        job.wrap = text::TextWrapping {
            max_width: 0.0,
            max_rows: 1,
            break_anywhere: true,
            overflow_character: Some('…'),
        };
        $ui.label(job);
    };
}

impl Application {
    fn new(ctx: &Context) -> Self {
        // make font bigger
        configure_fonts(ctx);

        let config = Config::get_config();

        Self {
            input_title: String::new(),
            input_url: String::new(),
            input_author: String::new(),
            input_published_date: NaiveDate::from(Local::now().naive_local()), // Current date
            input_published_enabled: false,
            input_viewed_date: NaiveDate::from(Local::now().naive_local()), // Current date
            curr_page: AppPage::Start,
            sources_cache: Arc::new(RwLock::new(vec![])),
            edit_windows_open: false,       // edit modal
            edit_source: Source::default(), // source to edit in the edit modal
            source_format: config.format_standard,
        }
    }

    // get input source from user
    fn get_source(&self) -> Source {
        Source {
            id: -1,
            url: self.input_url.clone(),
            author: self.input_author.clone(),
            date: self.input_published_date,
        }
    }

    // clears text fields and reset date to now
    fn clear_input(&mut self) {
        self.input_url.clear();
        self.input_author.clear();
        self.input_published_date = NaiveDate::from(Local::now().naive_local());
        self.input_viewed_date = NaiveDate::from(Local::now().naive_local());
    }

    fn update_source_cache(&self) {
        let sources = self.sources_cache.clone();
        tokio::task::spawn(async move {
            *sources.write().unwrap() = get_all_sources().await.expect("Error loading sources");
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
    // runs every frame
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
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
                    self.update_source_cache();
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
                AppPage::List => render_list_page(self, ui, ctx),
                AppPage::Settings => render_settings_page(self, ui),
            }
        });
    }
}

fn render_start_page(app: &mut Application, ui: &mut Ui) {
    Grid::new("SourceInput").num_columns(2).show(ui, |ui| {
        // input title
        let title_label = ui.label("Title:");
        ui.text_edit_singleline(&mut app.input_title)
            .labelled_by(title_label.id);
        ui.end_row();

        // input URL
        let url_label = ui.label("URL:");
        ui.text_edit_singleline(&mut app.input_url)
            .labelled_by(url_label.id);
        ui.end_row();

        // input author
        let author_label = ui.label("Author:");

        let input_author = TextEdit::singleline(&mut app.input_author)
            .hint_text("Leave empty if unknown");
        ui.add(input_author)
            .labelled_by(author_label.id);
        ui.end_row();

        // input published date
        let published_label = ui.label("Published on:");
        ui.horizontal(|ui| {
            ui.add_enabled(!app.input_published_enabled, DatePickerButton::new(&mut app.input_published_date)
                .id_source("InputPublishedDate") // needs to be set otherwise the UI would bug with multiple date pickers
                .show_icon(false))
                .labelled_by(published_label.id);
            ui.checkbox(&mut app.input_published_enabled, "Unknown");
        });

        ui.end_row();

        // input viewed date
        let viewed_label = ui.label("Viewed on:");
        ui.add(DatePickerButton::new(&mut app.input_viewed_date)
            .id_source("InputViewedDate") // needs to be set otherwise the UI would bug with multiple date pickers
            .show_icon(false))
            .labelled_by(viewed_label.id);
        ui.end_row();
    });

    ui.add_space(5.0);
    ui.separator();
    ui.add_space(5.0);

    ui.horizontal(|ui| {
        // save input source to DB
        if ui.button("Save").clicked() {
            handle_source_save(app);
        }

        // clear input
        if ui.button("Clear").clicked() {
            app.clear_input();
        }
    });
}

fn configure_fonts(ctx: &Context) {
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

fn render_list_page(app: &mut Application, ui: &mut Ui, ctx: &Context) {
    if ui.button("Copy all").clicked() {
        set_all_clipboard(&app.sources_cache.read().unwrap());
    }

    ui.add_space(10.0);

    render_sources(app, ui, ctx);
}

fn render_sources(app: &mut Application, ui: &mut Ui, ctx: &Context) {
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .drag_to_scroll(true)
        .scroll_bar_visibility(ScrollBarVisibility::AlwaysVisible)
        .show(ui, |ui| {
            if app.sources_cache.clone().read().unwrap().is_empty() {
                CentralPanel::default().show_inside(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Empty");
                    });
                });
                return;
            }

            #[allow(clippy::unnecessary_to_owned)]
            for source in app.sources_cache.clone().read().unwrap().to_vec() {
                // source preview
                ui.vertical(|ui| {
                    let id = format!("Index: {}", &source.id);
                    text_label_wrapped!(&id, ui);

                    let url = format!("URL: {}", &source.url);
                    text_label_wrapped!(&url, ui);

                    let author = format!("Author: {}", &source.author);
                    text_label_wrapped!(&author, ui);

                    let date = format!("Date: {}", &source.date.format("%d. %m. %Y"));
                    text_label_wrapped!(&date, ui);
                });

                ui.add_space(5.0);

                // buttons
                ui.horizontal(|ui| {
                    let copy_button = ui.button("Copy");
                    let edit_button = ui.button("Edit");
                    let delete_button = ui.button("Delete");

                    // copy one source
                    if copy_button.clicked() {
                        set_clipboard(&source);
                    }

                    // opens edit modal
                    if edit_button.clicked() {
                        //
                        app.edit_source = source.clone();
                        app.edit_windows_open = true;
                    }

                    let mut update_cache = false;

                    if app.edit_windows_open && app.edit_source.id == source.id {
                        // app.edit_source.id == source.id needed because else it would open an edit model x number of sources in the db

                        // needed because the borrow checker is fucking stupid
                        let mut window_open = true;

                        // edit modal
                        egui::Window::new("Edit source")
                            .collapsible(false)
                            .open(&mut window_open)
                            .show(ctx, |ui| {
                                Grid::new("SourceInput").num_columns(2).show(ui, |ui| {
                                    // input URL
                                    let url_label = ui.label("URL: ");
                                    ui.text_edit_multiline(&mut app.edit_source.url)
                                        .labelled_by(url_label.id);
                                    ui.end_row();

                                    // input author
                                    let author_label = ui.label("Author: ");
                                    ui.text_edit_singleline(&mut app.edit_source.author)
                                        .labelled_by(author_label.id);
                                    ui.end_row();

                                    // input date
                                    let date_label = ui.label("Date: ");
                                    ui.add(DatePickerButton::new(&mut app.edit_source.date))
                                        .labelled_by(date_label.id);
                                    ui.end_row();
                                });

                                ui.add_space(10.0);

                                if ui.button("Save").clicked() {
                                    handle_update_source(app.edit_source.id, &app.edit_source, app);
                                    update_cache = true;
                                    app.edit_windows_open = false;
                                }
                            });

                        if !window_open {
                            app.edit_windows_open = false;
                        }
                    }

                    if delete_button.clicked() {
                        handle_delete_source(source.id, app);
                        update_cache = true;
                    }

                    if update_cache {
                        app.update_source_cache();
                    }
                });

                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);
            }
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

// async delete source
fn handle_delete_source(id: i64, app: &Application) {
    let source_cache = app.sources_cache.clone();

    tokio::task::spawn(async move {
        delete_source(id).await.expect("Error deleting source");

        // update source cache
        *source_cache.write().unwrap() = get_all_sources().await.expect("Error loading sources");
    });
}

// async update source
fn handle_update_source(id: i64, source: &Source, app: &Application) {
    let source = source.clone();
    let source_cache = app.sources_cache.clone();

    tokio::task::spawn(async move {
        update_source(id, &source)
            .await
            .expect("Error deleting source");

        // update source cache
        *source_cache.write().unwrap() = get_all_sources().await.expect("Error loading sources");
    });
}

// async save source
fn handle_source_save(app: &Application) {
    let source = app.get_source();
    let source_cache = app.sources_cache.clone();

    tokio::task::spawn(async move {
        insert_source(&source)
            .await
            .expect("Error inserting source in database");

        // update source cache
        *source_cache.write().unwrap() = get_all_sources().await.expect("Error loading sources");
    });
}

fn render_settings_page(app: &mut Application, ui: &mut Ui) {
    // select source formatting standard
    ComboBox::from_label("Select source format")
        .selected_text(format!("{:?}", app.source_format))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut app.source_format,
                FormatStandard::Default,
                FormatStandard::Default.to_string(),
            );
            ui.selectable_value(
                &mut app.source_format,
                FormatStandard::IEEE,
                FormatStandard::IEEE.to_string(),
            );
            ui.selectable_value(
                &mut app.source_format,
                FormatStandard::APA,
                FormatStandard::APA.to_string(),
            );
        });

    ui.add_space(10.0);
    // Save button
    if ui.button("Save").clicked() {
        let mut config = Config::get_config();

        // Source formatting standard
        config.format_standard = app.source_format.clone();
        config.save();
    }
}
