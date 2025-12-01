use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub pages: Vec<ConfigPage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPage {
    pub name: String,
    pub layout: ConfigPageLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigPageLayout {
    Linear {
        /// Panels in order, and how much space each panel should take up.
        ///
        /// Space is calculated based on the sum of all weights, but the intention is to keep at
        /// sum = 100 so you can see them as percentages.
        panels: Vec<(i32, ConfigPanel)>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigPanel {
    Trackpad,
    ButtonGrid { rows: Vec<Vec<ConfigButton>> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigButton {
    pub label: String,
    pub command: RemoteCommand,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RemoteCommand {
    TrackpadMove { delta_x: i32, delta_y: i32 },
    TrackpadClick { button: TrackpadButton },
    TrackpadScroll { delta_x: i32, delta_y: i32 },

    CecImageViewOn,
    CecActiveSourceSelf,
    CecActiveSource { physical_address: [u8; 4] },
    CecStandby,

    MpdPlayPause,
    MpdNext,
    MpdPrevious,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TrackpadButton {
    Left,
    Right,
    Middle,
}
