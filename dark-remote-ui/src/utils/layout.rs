use egui::{Layout, UiBuilder};

pub struct SplitEqual {
    layout: Layout,
}

impl SplitEqual {
    pub fn vertical() -> Self {
        Self {
            layout: Layout::top_down(egui::Align::Center),
        }
    }

    pub fn horizontal() -> Self {
        Self {
            layout: Layout::left_to_right(egui::Align::Center),
        }
    }

    pub fn iterate<T>(
        self,
        ui: &mut egui::Ui,
        iter: impl IntoIterator<Item = T, IntoIter: ExactSizeIterator>,
        mut f: impl FnMut(&mut egui::Ui, T),
    ) {
        let iter = iter.into_iter();

        let total = ui.available_size()[self.layout.is_vertical() as usize];
        let count = iter.len() as f32;
        let size_per = total / count;

        let mut remaining = ui.available_rect_before_wrap();
        let rects = (0..iter.len())
            .map(|_| {
                let (a, b) = if self.layout.is_vertical() {
                    remaining.split_top_bottom_at_y(remaining.top() + size_per)
                } else {
                    remaining.split_left_right_at_x(remaining.left() + size_per)
                };
                remaining = b;
                a
            })
            .collect::<Vec<_>>();

        for (rect, item) in rects.into_iter().zip(iter) {
            ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
                f(ui, item);
            });
        }
    }
}
