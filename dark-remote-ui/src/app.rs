use dark_remote_common::config::{
    Config, ConfigButton, ConfigPage, ConfigPageLayout, ConfigPanel, RemoteCommand,
};
use egui::{CentralPanel, Color32, Frame, Layout, TopBottomPanel, UiBuilder};

use crate::utils::{all_widget_visuals, layout::SplitEqual};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    config: Config,
    current_page: String,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            config: Config {
                pages: vec![ConfigPage {
                    name: "Test".to_owned(),
                    layout: ConfigPageLayout::Linear {
                        panels: vec![
                            (60, ConfigPanel::Trackpad),
                            (
                                40,
                                ConfigPanel::ButtonGrid {
                                    rows: vec![
                                        vec![],
                                        vec![
                                            ConfigButton {
                                                label: "Prev".to_owned(),
                                                command: RemoteCommand::MpdPrevious,
                                            },
                                            ConfigButton {
                                                label: "Pause".to_owned(),
                                                command: RemoteCommand::MpdPlayPause,
                                            },
                                            ConfigButton {
                                                label: "Next".to_owned(),
                                                command: RemoteCommand::MpdNext,
                                            },
                                        ],
                                        vec![],
                                    ],
                                },
                            ),
                        ],
                    },
                }],
            },
            current_page: String::new(),
        }
    }
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default()
    }
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.all_styles_mut(|s| {
            s.visuals.panel_fill = if s.visuals.dark_mode {
                Color32::BLACK
            } else {
                Color32::WHITE
            };
        });

        TopBottomPanel::bottom("page selection").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for page in &self.config.pages {
                    if ui.button(&page.name).clicked() {
                        self.current_page = page.name.clone();
                    }
                }
            });
        });

        CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                all_widget_visuals(ui.style_mut(), |v| {
                    v.corner_radius = 16.0.into();
                });

                let Some(config_page) = self
                    .config
                    .pages
                    .iter()
                    .find(|p| p.name == self.current_page)
                else {
                    ui.label("No page selected");
                    return;
                };

                let ConfigPageLayout::Linear { panels } = &config_page.layout;

                ui.with_layout(Layout::top_down(egui::Align::Center), |ui| {
                    let weight_sum: f32 = panels.iter().map(|(weight, _)| *weight as f32).sum();

                    let total_height = ui.available_height();

                    for &(weight, ref panel) in panels {
                        let height = total_height * (weight as f32) / weight_sum;

                        let rect = ui.available_rect_before_wrap();
                        let rect = rect.split_top_bottom_at_y(rect.top() + height).0;

                        let res = {
                            ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                                match panel {
                                    ConfigPanel::Trackpad => {
                                        ui.painter().rect_filled(
                                            ui.available_rect_before_wrap().shrink(10.),
                                            50.,
                                            ui.visuals().widgets.noninteractive.bg_fill,
                                        );
                                    }
                                    ConfigPanel::ButtonGrid { rows } => {
                                        SplitEqual::vertical().iterate(ui, rows, |ui, buttons| {
                                            SplitEqual::horizontal().iterate(
                                                ui,
                                                buttons,
                                                |ui, button| {
                                                    let res = ui.place(
                                                        ui.available_rect_before_wrap().shrink(8.),
                                                        egui::Button::new(&button.label),
                                                    );

                                                    #[expect(clippy::print_stdout)]
                                                    if res.clicked() {
                                                        println!("Command: {:?}", button.command);
                                                    }
                                                },
                                            );
                                        });
                                    }
                                }

                                ui.take_available_space();
                            })
                        };

                        ui.advance_cursor_after_rect(res.response.rect);
                    }
                });
            });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
