use egui::{Style, style::WidgetVisuals};

pub mod layout;

pub fn all_widget_visuals(style: &mut Style, mut f: impl FnMut(&mut WidgetVisuals)) {
    for visuals in [
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.noninteractive,
        &mut style.visuals.widgets.open,
    ] {
        f(visuals);
    }
}
