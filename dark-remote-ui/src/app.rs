use dark_remote_common::config::{
    Config, ConfigButton, ConfigPage, ConfigPageLayout, ConfigPanel, RemoteCommand, TrackpadButton,
};
use egui::{CentralPanel, Color32, Frame, Layout, Sense, TopBottomPanel, UiBuilder, Vec2};

use crate::{
    connection::Connection,
    utils::{all_widget_visuals, layout::SplitEqual},
};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct DarkRemoteApp {
    config: Config,
    current_page: String,
    #[serde(skip)]
    connection: Option<Connection>,
    #[serde(skip)]
    message: String,
}

impl Default for DarkRemoteApp {
    fn default() -> Self {
        Self {
            config: Config { pages: vec![] },
            current_page: String::new(),
            connection: None,
            message: String::new(),
        }
    }
}

impl DarkRemoteApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut this = cc
            .storage
            .and_then(|storage| eframe::get_value::<Self>(storage, eframe::APP_KEY))
            .unwrap_or_default();

        this.config = Config {
            pages: vec![ConfigPage {
                name: "Test".to_owned(),
                layout: ConfigPageLayout::Linear {
                    panels: vec![
                        (60, ConfigPanel::Trackpad),
                        (
                            40,
                            ConfigPanel::ButtonGrid {
                                rows: vec![
                                    vec![
                                        ConfigButton {
                                            label: "Turn on".to_owned(),
                                            command: RemoteCommand::CecImageViewOn,
                                        },
                                        ConfigButton {
                                            label: "Switch".to_owned(),
                                            command: RemoteCommand::CecActiveSourceSelf,
                                        },
                                        ConfigButton {
                                            label: "Standby".to_owned(),
                                            command: RemoteCommand::CecStandby,
                                        },
                                    ],
                                    vec![
                                        ConfigButton {
                                            label: "HDMI 1".to_owned(),
                                            command: RemoteCommand::CecActiveSource {
                                                physical_address: [1, 0, 0, 0],
                                            },
                                        },
                                        ConfigButton {
                                            label: "HDMI 2".to_owned(),
                                            command: RemoteCommand::CecActiveSource {
                                                physical_address: [2, 0, 0, 0],
                                            },
                                        },
                                        ConfigButton {
                                            label: "HDMI 3".to_owned(),
                                            command: RemoteCommand::CecActiveSource {
                                                physical_address: [3, 0, 0, 0],
                                            },
                                        },
                                        ConfigButton {
                                            label: "HDMI 4".to_owned(),
                                            command: RemoteCommand::CecActiveSource {
                                                physical_address: [4, 0, 0, 0],
                                            },
                                        },
                                    ],
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
        };

        this
    }
}

impl eframe::App for DarkRemoteApp {
    #[expect(clippy::too_many_lines)] // TODO: later
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
                ui.add_space(30.);

                if ui.button("Connect").clicked() {
                    let res = Connection::new("ws://tetsuya.fbk.red:3000/ws");

                    match res {
                        Ok(conn) => self.connection = Some(conn),
                        Err(err) => self.message = format!("Failed to connect: {err}"),
                    }
                }

                for page in &self.config.pages {
                    if ui.button(&page.name).clicked() {
                        self.current_page = page.name.clone();
                    }
                }

                if let Some(msg) = self.connection.as_mut().and_then(|c| c.check_msg()) {
                    self.message = msg;
                }

                ui.label(&self.message);
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
                                        let rect = ui.available_rect_before_wrap().shrink(10.);

                                        let response =
                                            ui.allocate_rect(rect, Sense::click_and_drag());

                                        ui.painter().rect_filled(
                                            rect,
                                            50.,
                                            ui.visuals().widgets.noninteractive.bg_fill,
                                        );
                                        if let Some(conn) = &mut self.connection {
                                            let delta = response.drag_delta();
                                            if delta != Vec2::ZERO {
                                                let sensitivity =
                                                    (delta.length() / 10.).clamp(1., 5.);

                                                conn.send(RemoteCommand::TrackpadMove {
                                                    delta_x: (delta.x * sensitivity).round() as i32,
                                                    delta_y: (delta.y * sensitivity).round() as i32,
                                                });
                                            }

                                            if response.clicked() {
                                                conn.send(RemoteCommand::TrackpadClick {
                                                    button: TrackpadButton::Left,
                                                });
                                            }

                                            if response.long_touched() {
                                                conn.send(RemoteCommand::TrackpadClick {
                                                    button: TrackpadButton::Right,
                                                });
                                            }
                                        }
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

                                                    if let Some(conn) = &mut self.connection
                                                        && res.clicked()
                                                    {
                                                        conn.send(button.command);
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
