use crate::{data::appearance::Appearance, state::VoiceMember};
use gpui::{prelude::*, *};
use std::{
    collections::HashMap,
    io::{Cursor, Read},
    sync::Arc,
    time::Duration,
};

#[derive(Default)]
pub struct AvatarCache {
    images: HashMap<String, Option<Arc<Image>>>,
}
impl Global for AvatarCache {}

fn load_avatar(url: &str) -> anyhow::Result<Arc<Image>> {
    let client = reqwest::blocking::Client::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()?;
    let response = client.get(url).send()?.error_for_status()?;
    let mut bytes = Vec::new();
    response.take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
    anyhow::ensure!(bytes.len() <= 1024 * 1024, "Avatar exceeds 1 MiB");
    let mut reader = image::ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(2048);
    limits.max_image_height = Some(2048);
    reader.limits(limits);
    let decoded = reader.decode()?.thumbnail(128, 128);
    let mut png = Cursor::new(Vec::new());
    decoded.write_to(&mut png, image::ImageFormat::Png)?;
    Ok(Arc::new(Image::from_bytes(
        ImageFormat::Png,
        png.into_inner(),
    )))
}

fn avatar(url: &str, cx: &mut App) -> Option<Arc<Image>> {
    if url.len() > 2048 || !url.starts_with("https://") {
        return None;
    }
    if let Some(image) = cx.global::<AvatarCache>().images.get(url) {
        return image.clone();
    }
    let cache = cx.global_mut::<AvatarCache>();
    if cache.images.len() >= 256 {
        cache.images.clear();
    }
    cache.images.insert(url.to_string(), None);
    let url = url.to_string();
    let request_url = url.clone();
    let task = cx
        .background_executor()
        .spawn(async move { load_avatar(&request_url).ok() });
    cx.spawn(async move |cx| {
        let image = task.await;
        let _ = cx.update(|cx| {
            cx.global_mut::<AvatarCache>().images.insert(url, image);
            cx.refresh_windows();
        });
    })
    .detach();
    None
}

pub fn user_cell(
    member: &VoiceMember,
    mic_muted: bool,
    style: &Appearance,
    cx: &mut App,
) -> impl IntoElement {
    let scale = style.factor();
    let avatar_size = (if style.compact { 22.0 } else { 32.0 }) * scale;
    let name = if member.is_self {
        format!("{} (you)", member.name)
    } else {
        member.name.clone()
    };
    let initial = member
        .name
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_else(|| "?".into());
    let color =
        rgb((member.color.0 as u32) << 16 | (member.color.1 as u32) << 8 | member.color.2 as u32);
    let mut row = div()
        .flex()
        .items_center()
        .gap(px(8.0 * scale))
        .w_full()
        .h(px(style.row_height()))
        .flex_shrink_0()
        .text_size(px(14.0 * scale))
        .text_color(style.text());
    if style.show_avatars {
        let picture = member.avatar_url.as_deref().and_then(|url| avatar(url, cx));
        let inner = if let Some(picture) = picture {
            img(picture)
                .size(px(avatar_size))
                .rounded_full()
                .object_fit(ObjectFit::Cover)
                .into_any_element()
        } else {
            div()
                .size(px(avatar_size))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .bg(color)
                .text_color(rgb(0xffffff))
                .child(initial)
                .into_any_element()
        };
        row = row.child(
            div()
                .size(px(avatar_size + 4.0))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .border_2()
                .border_color(if member.speaking {
                    style.highlight()
                } else {
                    rgba(0x00000000)
                })
                .child(inner),
        );
    }
    if style.show_names {
        row = row.child(div().flex_1().min_w_0().text_ellipsis().child(name));
    }
    if member.muted || (member.is_self && mic_muted) {
        row = row.child(
            svg()
                .path("icons/microphone-off.svg")
                .size(px(14.0 * scale))
                .flex_shrink_0(),
        );
    } else if member.speaking && !style.show_avatars {
        row = row.child(div().size(px(6.0)).rounded_full().bg(style.highlight()));
    }
    row
}
