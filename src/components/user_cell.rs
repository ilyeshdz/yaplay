use freya::prelude::*;

#[derive(PartialEq)]
pub struct UserCell {
    pub(crate) image_color: Color,
    pub(crate) name: String,
}

impl Component for UserCell {
    fn render(&self) -> impl IntoElement {
        rect()
            .child(
                rect()
                    .background(self.image_color)
                    .rounded_full()
                    .width(Size::px(30.0))
                    .height(Size::px(30.0))
            )
            .child(label().text(self.name.clone()))
            .padding(Gaps::new_all(8.0))
            .color(Color::WHITE)
            .center()
            .spacing(10.0)
            .horizontal()
    }
}