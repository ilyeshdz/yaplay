use freya::prelude::*;

use crate::state::Notification;

pub fn notification_list(notifications: &[Notification]) -> impl IntoElement {
    let theme = use_theme();
    let colors = theme.read().colors.clone();

    let mut list = rect()
        .position(Position::new_global().top(48.0).right(24.0))
        .vertical()
        .spacing(8.0);

    for notification in notifications.iter().rev() {
        list = list.child(
            rect()
                .width(Size::px(320.0))
                .vertical()
                .spacing(4.0)
                .padding(Gaps::new_symmetric(12.0, 14.0))
                .background(Color::from_argb(215, 18, 18, 24))
                .rounded_lg()
                .border(Border::new().fill(colors.border).width(1.0))
                .child(
                    label()
                        .text(notification.title.clone())
                        .body()
                        .color(colors.text_primary),
                )
                .child(
                    label()
                        .text(notification.body.clone())
                        .color(colors.text_secondary),
                ),
        );
    }

    list
}
