use freya::prelude::*;

#[derive(PartialEq)]
pub struct UserCell {
    pub name: String,
    pub color: Color,
    pub speaking: bool,
    pub muted: bool,
    pub is_self: bool,
}

impl Component for UserCell {
    fn render(&self) -> impl IntoElement {
        let theme = use_theme();
        let colors = theme.read().colors.clone();

        let name = if self.is_self {
            format!("{} (you)", self.name)
        } else {
            self.name.clone()
        };
        let initial: String = self
            .name
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "?".to_string());

        let mut row = rect()
            .horizontal()
            .spacing(8.0)
            .cross_align(Alignment::Center)
            .child(
                rect()
                    .width(Size::px(30.0))
                    .height(Size::px(30.0))
                    .rounded_full()
                    .center()
                    .background(self.color)
                    .border(if self.speaking {
                        Border::new().fill(colors.success).width(2.0)
                    } else {
                        Border::new()
                    })
                    .child(label().text(initial).color(colors.text_inverse)),
            )
            .child(label().text(name).color(if self.muted {
                colors.text_secondary
            } else {
                colors.text_primary
            }));

        if self.muted {
            row = row.child(
                rect()
                    .width(Size::px(8.0))
                    .height(Size::px(8.0))
                    .rounded_full()
                    .background(colors.error),
            );
        }

        row
    }
}
