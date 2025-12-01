use axum::extract::ws::{self, WebSocket};
use cec_linux::{CecDevice, CecLogicalAddress, CecOpcode};
use core::time::Duration;
use dark_remote_common::config::{RemoteCommand, TrackpadButton};
use enigo::{Enigo, Mouse as _};
use std::net::SocketAddr;
use thiserror::Error;

pub struct Connection {
    client: SocketAddr,

    enigo: Enigo,
    cec: CecDevice,
}

impl Connection {
    pub fn new(client: SocketAddr) -> Self {
        let enigo = Enigo::new(&enigo::Settings::default()).expect("failed to initialize Enigo");

        let cec = CecDevice::open("/dev/cec0").expect("failed to open CEC device");

        Self { client, enigo, cec }
    }

    pub async fn handle(self, socket: WebSocket) {
        let client = self.client;
        match self.handle_inner(socket).await {
            Ok(()) => tracing::info!(?client, "disconnected by user"),
            Err(error) => tracing::error!(?client, ?error, "error occurred"),
        }
    }

    pub async fn handle_inner(mut self, mut socket: WebSocket) -> Result<(), axum::Error> {
        while let Some(msg) = socket.recv().await {
            let msg = match msg? {
                ws::Message::Text(utf8_bytes) => serde_json::from_str::<RemoteCommand>(&utf8_bytes),
                ws::Message::Binary(bytes) => {
                    // might use some binary format in the future
                    tracing::info!(?self.client, "received binary message of {} bytes", bytes.len());
                    continue;
                }
                ws::Message::Ping(_) | ws::Message::Pong(_) => continue,
                ws::Message::Close(close_frame) => {
                    tracing::info!(?self.client, ?close_frame, "received close frame");
                    return Ok(());
                }
            };

            let cmd = match msg {
                Ok(cmd) => cmd,
                Err(error) => {
                    tracing::warn!(?self.client, ?error, "failed to parse message");
                    continue;
                }
            };

            match self.execute(cmd).await {
                Ok(()) => {}
                Err(error) => {
                    tracing::warn!(?self.client, ?cmd, ?error, "failed to execute command");
                }
            }
        }

        Ok(())
    }

    async fn execute(&mut self, cmd: RemoteCommand) -> Result<(), Error> {
        tracing::debug!(?self.client, ?cmd, "executing command");
        match cmd {
            RemoteCommand::TrackpadMove { delta_x, delta_y } => {
                let half_x = delta_x / 2;
                let rem_x = delta_x % 2;
                let half_y = delta_y / 2;
                let rem_y = delta_y % 2;

                self.enigo
                    .move_mouse(half_x + rem_x, half_y + rem_y, enigo::Coordinate::Rel)?;

                std::thread::sleep(Duration::from_millis(1000 / 120));

                self.enigo
                    .move_mouse(half_x, half_y, enigo::Coordinate::Rel)?;
            }
            RemoteCommand::TrackpadClick { button } => self
                .enigo
                .button(enigo_button(button), enigo::Direction::Click)?,
            RemoteCommand::TrackpadScroll { delta_x, delta_y } => {
                if delta_y != 0 {
                    self.enigo.scroll(delta_y, enigo::Axis::Vertical)?;
                }
                if delta_x != 0 {
                    self.enigo.scroll(delta_x, enigo::Axis::Horizontal)?;
                }
            }
            RemoteCommand::CecImageViewOn => {
                tracing::info!(?self.client, "sending CEC image view on command");
                self.cec
                    .transmit(
                        CecLogicalAddress::Playback2,
                        CecLogicalAddress::UnregisteredBroadcast,
                        CecOpcode::ImageViewOn,
                    )
                    .expect("failed to send CEC image view on command");
            }
            RemoteCommand::CecActiveSourceSelf => {
                tracing::info!(?self.client, "setting active source to self");
                self.cec
                    .transmit_data(
                        CecLogicalAddress::Playback2,
                        CecLogicalAddress::UnregisteredBroadcast,
                        CecOpcode::ActiveSource,
                        &[0x20, 0x00], // 2.0.0.0
                    )
                    .expect("failed to send CEC active source command");
            }
            RemoteCommand::CecActiveSource { physical_address: [a, b, c, d] } => {
                tracing::info!(?self.client, "setting active source to self");
                self.cec
                    .transmit_data(
                        CecLogicalAddress::Playback2,
                        CecLogicalAddress::UnregisteredBroadcast,
                        CecOpcode::ActiveSource,
                        // 1, 2, 3, 4 -> 0x12, 0x34
                        &[((a & 0xf) << 4) | (b & 0xf), ((c & 0xf) << 4) | (d & 0xf)],
                    )
                    .expect("failed to send CEC active source command");
            }
            RemoteCommand::CecStandby => {
                tracing::info!(?self.client, "sending CEC standby command");
                self.cec
                    .transmit(
                        CecLogicalAddress::Playback2,
                        CecLogicalAddress::UnregisteredBroadcast,
                        CecOpcode::Standby,
                    )
                    .expect("failed to send CEC standby command");
            }
            RemoteCommand::MpdPlayPause | RemoteCommand::MpdNext | RemoteCommand::MpdPrevious => {
                tracing::error!(?cmd, "unimplemented");
            }
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
enum Error {
    #[error("Input error: {0}")]
    Input(#[from] enigo::InputError),
}

fn enigo_button(button: TrackpadButton) -> enigo::Button {
    match button {
        TrackpadButton::Left => enigo::Button::Left,
        TrackpadButton::Right => enigo::Button::Right,
        TrackpadButton::Middle => enigo::Button::Middle,
    }
}
