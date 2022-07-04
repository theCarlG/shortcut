use std::collections::BTreeMap;

use eframe::egui;
use eframe::egui::TextStyle;
use eframe::epaint::Color32;
use eframe::epaint::FontFamily;
use eframe::epaint::FontId;

pub(crate) fn setup_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    visuals.faint_bg_color = Color32::from_rgb(35, 38, 46);

    visuals.widgets.hovered.bg_fill = Color32::WHITE;
    visuals.widgets.hovered.fg_stroke.color = Color32::from_rgb(61, 68, 80);

    visuals.widgets.active.fg_stroke.color = Color32::from_rgb(61, 68, 80);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(61, 68, 80);

    visuals.override_text_color = Some(Color32::from_rgb(220, 220, 220));

    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(14, 20, 27);

    ctx.set_visuals(visuals);
}

pub(crate) fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Emoji Fonts
    fonts.font_data.insert(
        "emoji-icon-font".to_owned(),
        egui::FontData::from_static(include_bytes!("../resources/EmojiFont.ttf")),
    );
    fonts.font_data.insert(
        "noto-emoji-font".to_owned(),
        egui::FontData::from_static(include_bytes!("../resources/NotoEmojiRegular.ttf")),
    );

    fonts.font_data.insert(
        "Poppins-Regular".to_owned(),
        egui::FontData::from_static(include_bytes!("../resources/PoppinsLight.ttf")),
    );
    fonts.families.insert(
        FontFamily::Name("Poppins-400".into()),
        vec![
            "Poppins-Regular".into(),
            // Add emoji fonts as a fallback
            "noto-emoji-font".into(),
            "emoji-icon-font".into(),
        ],
    );

    fonts.font_data.insert(
        "Poppins-Medium".to_owned(),
        egui::FontData::from_static(include_bytes!("../resources/PoppinsRegular.ttf")),
    );
    fonts.families.insert(
        FontFamily::Name("Poppins-500".into()),
        vec![
            "Poppins-Medium".into(),
            // Add emoji fonts as a fallback
            "noto-emoji-font".into(),
            "emoji-icon-font".into(),
        ],
    );

    fonts.font_data.insert(
        "Poppins-SemiBold".to_owned(),
        egui::FontData::from_static(include_bytes!("../resources/PoppinsMedium.ttf")),
    );
    fonts.families.insert(
        FontFamily::Name("Poppins-600".into()),
        vec![
            "Poppins-SemiBold".into(),
            // Add emoji fonts as a fallback
            "noto-emoji-font".into(),
            "emoji-icon-font".into(),
        ],
    );

    fonts.families.insert(
        FontFamily::Name("Poppins-600".into()),
        vec![
            "Poppins-SemiBold".into(),
            // Add emoji fonts as a fallback
            "noto-emoji-font".into(),
            "emoji-icon-font".into(),
        ],
    );

    let mut text_styles = BTreeMap::new();

    text_styles.insert(
        TextStyle::Heading,
        FontId {
            family: FontFamily::Name("Poppins-600".into()),
            size: 35.,
        },
    );

    text_styles.insert(
        TextStyle::Body,
        FontId {
            family: FontFamily::Name("Poppins-400".into()),
            size: 24.,
        },
    );

    text_styles.insert(
        TextStyle::Button,
        FontId {
            family: FontFamily::Name("Poppins-400".into()),
            size: 22.,
        },
    );

    text_styles.insert(
        TextStyle::Monospace,
        FontId {
            family: FontFamily::Name("Poppins-400".into()),
            size: 22.,
        },
    );

    text_styles.insert(
        TextStyle::Small,
        FontId {
            family: FontFamily::Name("Poppins-400".into()),
            size: 18.,
        },
    );

    ctx.set_fonts(fonts);
    ctx.set_style(egui::Style {
        text_styles,
        ..Default::default()
    });
}
