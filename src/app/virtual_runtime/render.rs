//! Draw the maintained VT controller views with the actual app Presentation.
//! Geometry/cels come from the views; no textures, fonts or biology are synthesized.

use super::{DialogIssue, PageView, REQUIRED_FONT_STEMS, REQUIRED_IMAGE_IDS, RuntimeView};
use crate::app::{
    Presentation,
    help_runtime::{fill, font_text, image, image_box, outline},
};
use crate::virtual_ui::{
    ButtonView, Cel, DialogView, FishDetails, FishManagementView, FontId, ImageId, Layer,
    ProductPreview, Rect as UiRect, SCREEN, SetupView, Sprite, StoreView,
};
use macroquad::prelude::*;
use std::collections::HashMap;

/// Optional caller-loaded owned-install text, keyed by production biography IDs.
/// Missing entries are presented explicitly as numeric IDs; no retail prose table
/// is compiled into the runtime. The adapter neither loads nor generates this data.
#[derive(Default)]
pub(in crate::app) struct BiographyCatalog<'a> {
    pub hometowns: Option<&'a HashMap<u16, String>>,
    pub likes: Option<&'a HashMap<u16, String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::app) enum ResourceIssue {
    MissingImage(&'static str),
    MissingCells(&'static str),
    InvalidCel(&'static str, Cel),
    InvalidImage(ImageId),
    MissingFont(&'static str),
    InvalidText(&'static str),
}

/// Parent may fail its real preload explicitly before exposing these screens.
pub(in crate::app) fn validate_assets(presentation: &Presentation) -> Vec<ResourceIssue> {
    let mut issues = Vec::new();
    for id in REQUIRED_IMAGE_IDS {
        if !presentation.images.contains_key(*id) {
            issues.push(ResourceIssue::MissingImage(id));
        }
        if !presentation
            .image_cells
            .get(*id)
            .is_some_and(|(columns, rows)| *columns > 0 && *rows > 0)
        {
            issues.push(ResourceIssue::MissingCells(id));
        }
    }
    for id in REQUIRED_FONT_STEMS {
        if !presentation.fonts.contains_key(*id) {
            issues.push(ResourceIssue::MissingFont(id));
        }
    }
    issues
}

/// Draw order is the model's source-shaped order, with adapter chrome before
/// model dialogs and unavailable-dialog cancellation last. Returns exact asset
/// failures for the parent to surface/log; the visible banner is not a substitute.
pub(in crate::app) fn draw(
    presentation: &Presentation,
    view: &RuntimeView,
    catalog: &BiographyCatalog<'_>,
) -> Vec<ResourceIssue> {
    let mut renderer = Renderer {
        presentation,
        issues: Vec::new(),
    };
    let dialog = match &view.content {
        PageView::Store(store) => {
            renderer.store(store);
            store.dialog.as_ref()
        }
        PageView::Fish(fish) => {
            renderer.fish(fish, catalog);
            fish.dialog.as_ref()
        }
        PageView::Setup(setup) => {
            renderer.setup(setup);
            setup.dialog.as_ref()
        }
    };
    if let Some(issue) = view.dialog_issue {
        renderer.unavailable_dialog(issue);
    }
    for entry in &view.navigation {
        renderer.button(&entry.button, true);
    }
    if let Some(dialog) = dialog {
        renderer.dialog(dialog);
    }
    if !renderer.issues.is_empty() {
        fill(
            Rect::new(0.0, 456.0, 640.0, 24.0),
            rect(SCREEN),
            Color::new(0.0, 0.0, 0.0, 0.9),
        );
        draw_text(
            "Virtual Tank presentation unavailable",
            8.0,
            474.0,
            18.0,
            YELLOW,
        );
    }
    renderer.issues
}

struct Renderer<'a> {
    presentation: &'a Presentation,
    issues: Vec<ResourceIssue>,
}

impl Renderer<'_> {
    fn issue(&mut self, issue: ResourceIssue) {
        if !self.issues.contains(&issue) {
            self.issues.push(issue);
        }
    }

    fn source(&mut self, sprite: &Sprite) -> Option<(&Texture2D, Rect)> {
        let Some(id) = sprite.image.resource_key() else {
            self.issue(ResourceIssue::InvalidImage(sprite.image));
            return None;
        };
        let Some(texture) = self.presentation.images.get(id) else {
            self.issue(ResourceIssue::MissingImage(id));
            return None;
        };
        let width = texture.width();
        let height = texture.height();
        let source = match sprite.cel {
            Cel::Whole => Rect::new(0.0, 0.0, width, height),
            Cel::Indexed(_) | Cel::Grid { .. } => {
                let Some(&(columns, rows)) = self
                    .presentation
                    .image_cells
                    .get(id)
                    .filter(|(columns, rows)| *columns > 0 && *rows > 0)
                else {
                    self.issue(ResourceIssue::MissingCells(id));
                    return None;
                };
                let (column, row) = match sprite.cel {
                    Cel::Indexed(index) => (u32::from(index) % columns, u32::from(index) / columns),
                    Cel::Grid { column, row } => (u32::from(column), u32::from(row)),
                    _ => unreachable!(),
                };
                if column >= columns || row >= rows {
                    self.issue(ResourceIssue::InvalidCel(id, sprite.cel));
                    return None;
                }
                Rect::new(
                    column as f32 * width / columns as f32,
                    row as f32 * height / rows as f32,
                    width / columns as f32,
                    height / rows as f32,
                )
            }
            Cel::Horizontal { index, count } if count > 0 && index < count => Rect::new(
                f32::from(index) * width / f32::from(count),
                0.0,
                width / f32::from(count),
                height,
            ),
            Cel::Cell {
                column,
                row,
                width,
                height,
            } => Rect::new(
                f32::from(column) * f32::from(width),
                f32::from(row) * f32::from(height),
                f32::from(width),
                f32::from(height),
            ),
            Cel::Horizontal { .. } => {
                self.issue(ResourceIssue::InvalidCel(id, sprite.cel));
                return None;
            }
        };
        if source.w <= 0.0
            || source.h <= 0.0
            || source.x + source.w > width
            || source.y + source.h > height
        {
            self.issue(ResourceIssue::InvalidCel(id, sprite.cel));
            return None;
        }
        Some((texture, source))
    }

    fn sprite(&mut self, sprite: &Sprite) {
        if let Some((texture, source)) = self.source(sprite) {
            image(
                texture,
                source,
                rect(sprite.bounds),
                rect(sprite.clip),
                Color::new(1.0, 1.0, 1.0, sprite.alpha.clamp(0.0, 1.0)),
                false,
            );
        }
    }

    fn preview(&mut self, preview: &ProductPreview) {
        self.sprite(match preview {
            ProductPreview::Fish(fish) => &fish.sprite,
            ProductPreview::Item(item) => item,
        });
    }

    fn button(&mut self, button: &ButtonView, label: bool) {
        self.sprite(&button.skin());
        if label {
            self.text(
                button.font,
                &button.text,
                inset(button.bounds, 4.0),
                if button.enabled { WHITE } else { GRAY },
                true,
            );
        }
    }

    fn title(&mut self, title: &str, font: FontId) {
        let bounds = UiRect::new(174.0, 0.0, 292.0, 39.0);
        self.sprite(&whole(ImageId::ScreenTitle, bounds, Layer::Overlay));
        self.text(font, title, bounds, WHITE, true);
    }

    fn store(&mut self, view: &StoreView) {
        // W1 places the Store chrome over widget bodies and fish, then prices,
        // highlights and speech. Sorting all overlays together would lose this.
        for slot in &view.slots {
            self.button(&slot.button, false);
            if let Some(preview) = &slot.preview {
                self.preview(preview);
            }
        }
        self.sprite(&view.background);
        for slot in &view.slots {
            if let Some(highlight) = &slot.highlight {
                self.sprite(highlight);
            }
            let mut bounds = slot.price_bounds;
            if let Some(shell) = &slot.shell_icon {
                self.sprite(shell);
                bounds.x += 20.0;
                bounds.width -= 20.0;
            }
            let color =
                if slot.price.is_some() && (!slot.affordable || !slot.fish_capacity_available) {
                    Color::from_rgba(255, 135, 110, 255)
                } else {
                    WHITE
                };
            self.text(slot.price_font, &slot.price_label, bounds, color, true);
        }
        self.text(
            FontId::Price,
            &format!("Shells: {}", view.shell_balance),
            UiRect::new(158.0, 440.0, 305.0, 32.0),
            WHITE,
            true,
        );
        if let Some(bubble) = &view.speech_bubble {
            self.sprite(bubble);
        }
        if let Some(speech) = &view.speech {
            self.text(view.speech_font, speech, view.speech_bounds, BLACK, false);
        }
        self.button(&view.return_button, true);
    }

    fn fish(&mut self, view: &FishManagementView, catalog: &BiographyCatalog<'_>) {
        self.sprite(&view.background);
        self.sprite(&view.panel);
        self.title(view.title, view.title_font);
        for row in &view.rows {
            self.button(&row.button, false);
            if let Some(preview) = &row.preview {
                self.sprite(&preview.sprite);
            }
        }
        for row in &view.rows {
            self.sprite(&row.ring);
            if let Some(reflection) = &row.reflection {
                self.sprite(reflection);
            }
            if let Some(name) = &row.name {
                self.text(
                    row.name_font,
                    name,
                    UiRect::new(
                        row.button.bounds.x + 3.0,
                        row.button.bounds.y + 63.0,
                        84.0,
                        20.0,
                    ),
                    WHITE,
                    true,
                );
            }
            if row.hidden {
                self.text(
                    FontId::Body,
                    "HIDDEN",
                    UiRect::new(
                        row.button.bounds.x + 3.0,
                        row.button.bounds.y + 35.0,
                        84.0,
                        22.0,
                    ),
                    YELLOW,
                    true,
                );
            }
        }
        if let Some(selected) = &view.selected {
            self.details(selected, catalog);
        }
        if let Some(guidance) = view.empty_guidance {
            self.text(
                FontId::Body,
                guidance,
                UiRect::new(232.0, 175.0, 175.0, 110.0),
                WHITE,
                false,
            );
        }
        for button in &view.controls {
            self.button(button, true);
        }
    }

    fn details(&mut self, details: &FishDetails, catalog: &BiographyCatalog<'_>) {
        self.sprite(&details.preview.sprite);
        self.text(
            details.name_font,
            &details.target.0.name,
            UiRect::new(230.0, 55.0, 180.0, 34.0),
            WHITE,
            true,
        );
        let mut catalog_missing = false;
        let hometown = catalog_label(catalog.hometowns, details.hometown, &mut catalog_missing);
        let likes = details
            .likes
            .iter()
            .map(|id| catalog_label(catalog.likes, *id, &mut catalog_missing))
            .collect::<Vec<_>>()
            .join(", ");
        let mut biography = format!(
            "{:?} / {:?}\n{}: {} Unix s\nHome: {hometown}\nLikes: {likes}\nMood ID: {}\nFeeds today: {}",
            details.target.0.kind,
            details.target.0.size,
            if details.date_is_birth {
                "Born"
            } else {
                "Purchased"
            },
            details.timestamp_seconds,
            details.mental_state,
            details.feeds_today
        );
        if let Some(interest) = details.special_interest {
            biography.push_str(&format!("\nInterest: {interest}"));
        }
        if !details.trait_labels.is_empty() {
            biography.push_str(&format!("\n{}", details.trait_labels.join(", ")));
        }
        if catalog_missing {
            biography.push_str("\nBiography text unavailable.");
        }
        self.text(
            details.body_font,
            &biography,
            details.bounds,
            Color::new(1.0, 1.0, 1.0, details.biography_alpha.clamp(0.0, 1.0)),
            false,
        );
        if details.finance_visible {
            let purchase = details.purchase_price.map_or_else(
                || "Not purchased".to_owned(),
                |price| format!("Purchase: {price} shells"),
            );
            let sale = details.sale_value.map_or_else(
                || "Sale unavailable".to_owned(),
                |price| format!("Sale: {price} shells"),
            );
            self.text(
                details.body_font,
                &format!("{purchase}\n{sale}"),
                details.bounds,
                Color::new(1.0, 1.0, 1.0, details.finance_alpha.clamp(0.0, 1.0)),
                false,
            );
        }
    }

    fn setup(&mut self, view: &SetupView) {
        self.sprite(&view.background);
        self.title(view.title, view.title_font);
        self.sprite(&view.backdrop_frame);
        if let Some(backdrop) = &view.backdrop {
            self.sprite(backdrop);
        }
        self.text(
            FontId::Heading,
            &format!("Backdrop {}", u16::from(view.backdrop_index) + 1),
            UiRect::new(65.0, 110.0, 220.0, 27.0),
            WHITE,
            true,
        );
        for button in &view.controls {
            self.button(button, true);
        }
        for setting in &view.settings {
            self.button(&setting.button, false);
            self.text(
                setting.button.font,
                &setting.button.text,
                setting.label_bounds,
                WHITE,
                false,
            );
        }
        self.text(
            view.guidance_font,
            view.guidance,
            view.guidance_bounds,
            WHITE,
            false,
        );
        if !view.unavailable_devices.is_empty() {
            self.text(
                FontId::Body,
                "Additional tank devices are available in Store.",
                UiRect::new(340.0, 370.0, 270.0, 45.0),
                GRAY,
                false,
            );
        }
    }

    fn dialog_panel(&mut self, bounds: UiRect) {
        fill(rect(SCREEN), rect(SCREEN), Color::new(0.0, 0.0, 0.0, 0.55));
        let Some(texture) = self.presentation.images.get("IMAGE_DIALOG") else {
            self.issue(ResourceIssue::MissingImage("IMAGE_DIALOG"));
            return;
        };
        image_box(
            texture,
            Rect::new(0.0, 0.0, texture.width(), texture.height()),
            rect(bounds),
            rect(SCREEN),
        );
    }

    fn dialog(&mut self, dialog: &DialogView) {
        self.dialog_panel(dialog.bounds);
        self.text(
            dialog.title_font,
            dialog.title,
            UiRect::new(
                dialog.bounds.x + 15.0,
                dialog.bounds.y + 10.0,
                dialog.bounds.width - 30.0,
                30.0,
            ),
            WHITE,
            true,
        );
        if let Some(preview) = &dialog.preview {
            self.preview(preview);
        }
        if let Some(price) = dialog.price {
            self.text(
                FontId::Price,
                &format!("{price} shells"),
                UiRect::new(375.0, 188.0, 165.0, 45.0),
                WHITE,
                true,
            );
        }
        let mut guidance = dialog.text_bounds;
        if dialog.name.is_some() {
            guidance.height = 32.0;
        }
        self.text(
            dialog.body_font,
            &dialog.guidance,
            guidance,
            if dialog.stale { YELLOW } else { WHITE },
            false,
        );
        if let Some(name) = &dialog.name {
            let bounds = UiRect::new(135.0, 279.0, 370.0, 25.0);
            fill(
                rect(bounds),
                rect(dialog.bounds),
                Color::new(0.0, 0.0, 0.0, 0.7),
            );
            outline(
                rect(bounds),
                rect(dialog.bounds),
                1.0,
                if dialog.stale { GRAY } else { YELLOW },
            );
            self.text(
                dialog.body_font,
                if name.is_empty() {
                    "Enter fish name"
                } else {
                    name
                },
                inset(bounds, 3.0),
                if name.is_empty() { GRAY } else { WHITE },
                false,
            );
        }
        for button in &dialog.buttons {
            self.button(button, true);
        }
    }

    fn unavailable_dialog(&mut self, issue: DialogIssue) {
        self.dialog_panel(UiRect::new(85.0, 140.0, 470.0, 230.0));
        self.text(
            FontId::Heading,
            "Selection unavailable",
            UiRect::new(100.0, 150.0, 440.0, 35.0),
            WHITE,
            true,
        );
        let guidance = match issue {
            DialogIssue::MissingCapture => {
                "Original selection was not captured. Cancel, then choose again."
            }
            DialogIssue::PhaseChanged => "Dialog selection changed. Cancel, then choose again.",
        };
        self.text(
            FontId::Body,
            guidance,
            UiRect::new(120.0, 215.0, 400.0, 75.0),
            YELLOW,
            false,
        );
    }

    fn text(&mut self, id: FontId, text: &str, bounds: UiRect, color: Color, centered: bool) {
        if color.a <= 0.0 || text.is_empty() {
            return;
        }
        let key = id.catalog_key();
        let Some(font) = self.presentation.fonts.get(key) else {
            self.issue(ResourceIssue::MissingFont(key));
            return;
        };
        let measure = |text: &str| {
            font.metrics
                .measure_width(text)
                .ok()
                .map(|width| width as f32)
        };
        let Some(lines) = wrap_text(text, bounds.width, measure) else {
            self.issue(ResourceIssue::InvalidText(key));
            return;
        };
        let ascent = font
            .metrics
            .layers
            .iter()
            .map(|layer| layer.ascent)
            .max()
            .unwrap_or(1)
            .max(1) as f32;
        let height = font
            .metrics
            .layers
            .iter()
            .flat_map(|layer| layer.glyphs.values())
            .filter_map(|glyph| glyph.rect)
            .map(|rect| rect.height)
            .max()
            .unwrap_or(1)
            .max(1) as f32;
        let line_height = (height + 2.0).max(ascent + 2.0);
        let mut baseline = bounds.y
            + ascent
            + if centered {
                ((bounds.height - line_height * lines.len() as f32) / 2.0).max(0.0)
            } else {
                0.0
            };
        for line in &lines {
            let x = bounds.x
                + if centered {
                    ((bounds.width - measure(line).unwrap_or(bounds.width)) / 2.0).max(0.0)
                } else {
                    0.0
                };
            font_text(font, line, x, baseline, color, rect(bounds));
            baseline += line_height;
            if baseline - ascent >= bounds.y + bounds.height {
                break;
            }
        }
    }
}

fn rect(value: UiRect) -> Rect {
    Rect::new(value.x, value.y, value.width, value.height)
}
fn inset(value: UiRect, amount: f32) -> UiRect {
    UiRect::new(
        value.x + amount,
        value.y + amount,
        (value.width - 2.0 * amount).max(0.0),
        (value.height - 2.0 * amount).max(0.0),
    )
}
fn whole(image: ImageId, bounds: UiRect, layer: Layer) -> Sprite {
    Sprite {
        image,
        bounds,
        cel: Cel::Whole,
        clip: SCREEN,
        layer,
        alpha: 1.0,
    }
}

fn catalog_label(catalog: Option<&HashMap<u16, String>>, id: u16, missing: &mut bool) -> String {
    catalog
        .and_then(|catalog| catalog.get(&id))
        .cloned()
        .unwrap_or_else(|| {
            *missing = true;
            format!("#{id}")
        })
}

fn wrap_text(text: &str, width: f32, measure: impl Fn(&str) -> Option<f32>) -> Option<Vec<String>> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_owned()
            } else {
                format!("{current} {word}")
            };
            if measure(&candidate)? > width && !current.is_empty() {
                lines.push(current);
                current = word.to_owned();
            } else {
                current = candidate;
            }
        }
        lines.push(current);
    }
    Some(lines)
}
