//! Fixed coordinates and styles from the pre-extraction paint sites at a55f50a.
//! These offline checks do not assert retail or GPU-rendered pixel equivalence.

use macroquad::prelude::{Color, Rect, WHITE, YELLOW, vec2};
use turbofish_deluxe::{
    font::BitmapFont,
    highscores::screen::{self as hall, Control},
    ui::{self, FALLBACK_BUTTON_STYLE},
};

fn rect_tuple(bounds: Rect) -> (f32, f32, f32, f32) {
    (bounds.x, bounds.y, bounds.w, bounds.h)
}

#[test]
fn main_and_hall_button_labels_keep_their_geometry() {
    // Hall's Menu geometry is the same established MainButton destination.
    let menu = hall::control_bounds(Control::Menu, 30.0);
    assert_eq!(
        (menu.x, menu.y, menu.width, menu.height),
        (525.0, 4.0, 80.0, 30.0)
    );
    let bounds = Rect::new(menu.x, menu.y, menu.width, menu.height);
    assert_eq!(ui::centered_text_x(bounds, 40.0), 545.0);
    // Unclamped centering is intentional in Records, even for a wide label.
    assert_eq!(ui::centered_text_x(bounds, 100.0), 515.0);
    assert_eq!(ui::centered_text_x(bounds, 0.0), 565.0);
}

#[test]
fn story_controls_keep_ascent_centering_and_unicode_widths() {
    // Synthetic, project-owned font: two glyphs with distinct widths/kerning.
    let font = BitmapFont::parse(
        "CreateLayer Main;\n\
         LayerSetImage Main 'synthetic';\n\
         LayerSetPointSize Main 10;\n\
         LayerSetAscent Main 8;\n\
         LayerSetCharWidths Main ('A', 'é') (6, 10);\n\
         LayerSetKerningPairs Main ('Aé') (-2);\n\
         SetDefaultPointSize 10;",
    )
    .unwrap();
    let width = font.measure_width("Aé").unwrap() as f32;
    assert_eq!(width, 14.0);
    let bounds = Rect::new(190.0, 15.0, 30.0, 10.0);
    assert_eq!(ui::centered_text_x(bounds, width), 198.0);
    assert_eq!(ui::centered_ascent_baseline(bounds, 8.0), 24.0);
    // Records previously used zero width on an unavailable glyph; keep that
    // fallback caller-owned instead of changing font admission in the helper.
    let missing_width = font.measure_width("?").unwrap_or(0) as f32;
    assert_eq!(ui::centered_text_x(bounds, missing_width), 205.0);
}

#[test]
fn virtual_title_and_button_labels_keep_the_overflow_policy() {
    let title = Rect::new(174.0, 0.0, 292.0, 39.0);
    assert_eq!(ui::centered_text_x_clamped(title, 120.0), 260.0);
    assert_eq!(ui::text_block_baseline(title, 17.0, 20.0, 1, true), 26.5);

    let label = ui::inset(Rect::new(100.0, 50.0, 88.0, 32.0), 4.0);
    assert_eq!(rect_tuple(label), (104.0, 54.0, 80.0, 24.0));
    assert_eq!(ui::centered_text_x_clamped(label, 40.0), 124.0);
    assert_eq!(ui::centered_text_x_clamped(label, 100.0), 104.0);
    assert_eq!(ui::centered_text_x(label, 100.0), 94.0);
    // VT's measurement-failure fallback uses the available width, not zero.
    assert_eq!(ui::centered_text_x_clamped(label, label.w), 104.0);
}

#[test]
fn virtual_multiline_baselines_keep_top_alignment_when_full() {
    let bounds = Rect::new(120.0, 215.0, 400.0, 75.0);
    assert_eq!(ui::text_block_baseline(bounds, 12.0, 16.0, 2, true), 248.5);
    assert_eq!(ui::text_block_baseline(bounds, 12.0, 16.0, 2, false), 227.0);
    assert_eq!(ui::text_block_baseline(bounds, 12.0, 16.0, 5, true), 227.0);
    assert_eq!(ui::text_block_baseline(bounds, 12.0, 16.0, 0, true), 264.5);
}

#[test]
fn text_insets_keep_original_origin_and_zero_clamped_dimensions() {
    assert_eq!(
        rect_tuple(ui::inset(Rect::new(135.0, 279.0, 370.0, 25.0), 3.0)),
        (138.0, 282.0, 364.0, 19.0)
    );
    assert_eq!(
        rect_tuple(ui::inset(Rect::new(10.0, 20.0, 6.0, 4.0), 4.0)),
        (14.0, 24.0, 0.0, 0.0)
    );
}

#[test]
fn fallback_button_keeps_its_exact_skin_and_label_style() {
    let style = FALLBACK_BUTTON_STYLE;
    assert_eq!(style.fill, Color::new(0.05, 0.2, 0.26, 0.96));
    assert_eq!(style.outline, YELLOW);
    assert_eq!(style.outline_width, 1.0);
    assert_eq!(style.label_color, WHITE);
    assert_eq!(style.font_size, 18.0);
    assert_eq!(style.label_inset, 8.0);
    assert_eq!(style.baseline_fraction, 0.7);
    assert_eq!(
        style.label_position(Rect::new(525.0, 4.0, 80.0, 30.0)),
        vec2(533.0, 25.0)
    );
}

#[test]
fn screen_centered_labels_preserve_fixed_center_subtraction() {
    assert_eq!(ui::centered_label_x(320.0, 120.0), 260.0);
    assert_eq!(ui::centered_label_x(320.0, 700.0), -30.0);
    assert_eq!(ui::centered_label_x(320.0, 0.0), 320.0);
    assert_eq!(
        ui::centered_label_x(-0.0, 0.0).to_bits(),
        (-0.0_f32).to_bits()
    );
}
