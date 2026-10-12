use macroquad::math::Rect;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use turbofish_deluxe::font::{BitmapFont, Glyph, GlyphPlacement, GlyphRect};

const SCRIPT: &str = r#"
Define Letters ('A', 'V');
Define Widths (5, 6);
Define Boxes ((0, 0, 5, 8), (5, 0, 6, 8));
Define Offsets ((1, 0), (0, -1));
Define Pairs ("AV");
Define Values (-2);
CreateLayer Main;
LayerSetImage Main 'tiny';
LayerSetAscent Main 7;
LayerSetCharWidths Main Letters Widths;
LayerSetCharWidths Main (' ') (3);
LayerSetImageMap Main Letters Boxes;
LayerSetCharOffsets Main Letters Offsets;
LayerSetKerningPairs Main Pairs Values;
LayerSetAscentPadding Main 1;
LayerSetLineSpacingOffset Main -1;
LayerSetPointSize Main 9;
SetDefaultPointSize 9;
"#;

#[test]
fn interprets_glyph_metrics_kerning_and_image_reference() {
    let font = BitmapFont::parse(SCRIPT).expect("small declarative script");
    assert_eq!(font.default_point_size, 9);
    assert_eq!(font.layers.len(), 1);
    let layer = &font.layers[0];
    assert_eq!(layer.image_path, PathBuf::from("data/tiny"));
    assert_eq!(
        (
            layer.ascent,
            layer.ascent_padding,
            layer.line_spacing_offset
        ),
        (7, 1, -1)
    );
    assert_eq!(
        layer.glyphs[&'A'].rect,
        Some(GlyphRect {
            x: 0,
            y: 0,
            width: 5,
            height: 8
        })
    );
    assert_eq!(
        (layer.glyphs[&'A'].offset_x, layer.glyphs[&'V'].offset_y),
        (1, -1)
    );
    assert_eq!(font.measure_width("AV").unwrap(), 9);
    assert_eq!(font.measure_width("A V").unwrap(), 14);
    assert!(font.measure_width("X").is_err());
    layer.validate_atlas(11, 8).unwrap();
    assert!(
        layer
            .validate_atlas(10, 8)
            .unwrap_err()
            .to_string()
            .contains("outside atlas")
    );
}

#[test]
fn rejects_unrecognized_commands_mismatched_lists_and_unsafe_images() {
    assert!(
        BitmapFont::parse(&SCRIPT.replace("LayerSetAscent Main 7", "LayerSetMagic Main 7"))
            .unwrap_err()
            .to_string()
            .contains("unsupported command")
    );
    assert!(
        BitmapFont::parse(&SCRIPT.replace("Define Widths (5, 6)", "Define Widths (5)"))
            .unwrap_err()
            .to_string()
            .contains("list length mismatch")
    );
    assert!(
        BitmapFont::parse(&SCRIPT.replace("'tiny'", "'../outside'"))
            .unwrap_err()
            .to_string()
            .contains("unsafe font path")
    );
}

#[test]
fn load_rejects_escaping_script_name() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("turbofish-font-{nonce}"));
    fs::create_dir_all(root.join("data")).unwrap();
    fs::write(root.join("data/tiny.txt"), SCRIPT).unwrap();
    assert_eq!(
        BitmapFont::load(&root, "tiny")
            .unwrap()
            .measure_width("AV")
            .unwrap(),
        9
    );
    assert!(
        BitmapFont::load(&root, "../outside")
            .unwrap_err()
            .to_string()
            .contains("unsafe")
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn loads_single_byte_western_glyph_without_lossy_replacement() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("turbofish-font-legacy-{nonce}"));
    fs::create_dir_all(root.join("data")).unwrap();
    let mut encoded = b"Define Chars ('".to_vec();
    encoded.push(0xe9); // e-acute in the owned scripts' single-byte encoding.
    encoded.extend_from_slice(b"'); Define Widths (5); Define Boxes ((0,0,5,8)); CreateLayer Main; LayerSetImage Main 'tiny'; LayerSetAscent Main 7; LayerSetCharWidths Main Chars Widths; LayerSetImageMap Main Chars Boxes; LayerSetPointSize Main 8; SetDefaultPointSize 8;");
    fs::write(root.join("data/tiny.txt"), &encoded).unwrap();
    assert_eq!(
        BitmapFont::load(&root, "tiny")
            .unwrap()
            .measure_width("é")
            .unwrap(),
        5
    );
    encoded.push(0x81);
    fs::write(root.join("data/tiny.txt"), encoded).unwrap();
    assert!(
        BitmapFont::load(&root, "tiny")
            .unwrap_err()
            .to_string()
            .contains("undefined Windows-1252")
    );
    fs::remove_dir_all(root).unwrap();
}

// These synthetic metrics and fixed coordinates specify the existing draw
// contract independently of the shared placement implementation. No atlas or
// native graphics context is used.
fn layout_font() -> BitmapFont {
    let mut font = BitmapFont::parse(SCRIPT).unwrap();
    let layer = &mut font.layers[0];
    layer.spacing = 2;
    layer.kerning.extend([
        (('V', 'A'), 1),
        (('A', '?'), 4),
        (('?', 'V'), 99),
        (('A', ' '), -1),
        ((' ', 'V'), 2),
    ]);
    font
}

fn placed(font: &BitmapFont, text: &str) -> Vec<GlyphPlacement> {
    let mut glyphs = Vec::new();
    font.for_each_glyph(text, 10.25, 100.5, |glyph| glyphs.push(glyph));
    glyphs
}

const A_SOURCE: GlyphRect = GlyphRect {
    x: 0,
    y: 0,
    width: 5,
    height: 8,
};
const V_SOURCE: GlyphRect = GlyphRect {
    x: 5,
    y: 0,
    width: 6,
    height: 8,
};

fn placement(layer_index: usize, source: GlyphRect, x: f32, y: f32) -> GlyphPlacement {
    GlyphPlacement {
        layer_index,
        source,
        x,
        y,
    }
}

#[test]
fn glyph_layout_applies_offsets_baseline_spacing_and_pair_kerning() {
    assert_eq!(
        placed(&layout_font(), "AVA"),
        [
            placement(0, A_SOURCE, 11.25, 93.5),
            placement(0, V_SOURCE, 15.25, 92.5),
            placement(0, A_SOURCE, 25.25, 93.5),
        ]
    );
}

#[test]
fn glyph_layout_skips_unknown_without_bridging_kerning() {
    let font = layout_font();
    // A's kerning toward '?' applies, but '?' contributes no advance or '?V'
    // kerning. The supported A/V pair is not adjacent in this input.
    assert_eq!(
        placed(&font, "A?V"),
        [
            placement(0, A_SOURCE, 11.25, 93.5),
            placement(0, V_SOURCE, 21.25, 92.5),
        ]
    );
    assert_eq!(placed(&font, "?V"), [placement(0, V_SOURCE, 10.25, 92.5)]);
    assert!(placed(&font, "???").is_empty());
    assert!(placed(&font, "").is_empty());
}

#[test]
fn glyph_layout_advances_rectangle_less_glyphs() {
    assert_eq!(
        placed(&layout_font(), "A V"),
        [
            placement(0, A_SOURCE, 11.25, 93.5),
            placement(0, V_SOURCE, 23.25, 92.5),
        ]
    );
}

#[test]
fn glyph_layout_keeps_layer_order_and_independent_metrics() {
    let mut font = layout_font();
    let outline = BitmapFont::parse(
        "CreateLayer Outline;
         LayerSetImage Outline 'outline';
         LayerSetAscent Outline 9;
         LayerSetSpacing Outline -1;
         LayerSetPointSize Outline 13;
         LayerSetCharWidths Outline ('A', 'V') (8, 4);
         LayerSetImageMap Outline ('A', 'V') ((20, 3, 6, 9), (26, 3, 4, 9));
         LayerSetCharOffsets Outline ('A', 'V') ((-2, 1), (3, 0));
         LayerSetKerningPairs Outline ('AV') (-3);
         SetDefaultPointSize 13;",
    )
    .unwrap();
    font.layers.extend(outline.layers);
    assert_eq!(
        placed(&font, "AV"),
        [
            placement(0, A_SOURCE, 11.25, 93.5),
            placement(0, V_SOURCE, 15.25, 92.5),
            placement(
                1,
                GlyphRect {
                    x: 20,
                    y: 3,
                    width: 6,
                    height: 9,
                },
                8.25,
                92.5,
            ),
            placement(
                1,
                GlyphRect {
                    x: 26,
                    y: 3,
                    width: 4,
                    height: 9,
                },
                17.25,
                91.5,
            ),
        ]
    );
}

#[test]
fn glyph_layout_keeps_zero_area_rectangles_and_unicode() {
    let mut font = layout_font();
    let empty_source = GlyphRect {
        x: 11,
        y: 0,
        width: 0,
        height: 0,
    };
    let accent_source = GlyphRect {
        x: 11,
        y: 0,
        width: 7,
        height: 8,
    };
    font.layers[0].glyphs.extend([
        (
            'Z',
            Glyph {
                advance: 4,
                rect: Some(empty_source),
                ..Default::default()
            },
        ),
        (
            'é',
            Glyph {
                advance: 7,
                rect: Some(accent_source),
                offset_x: -1,
                offset_y: 2,
            },
        ),
    ]);
    assert_eq!(
        placed(&font, "A ZéV"),
        [
            placement(0, A_SOURCE, 11.25, 93.5),
            placement(0, empty_source, 21.25, 93.5),
            placement(0, accent_source, 26.25, 95.5),
            placement(0, V_SOURCE, 36.25, 92.5),
        ]
    );
}

#[test]
fn clipped_consumer_keeps_unclipped_glyph_positions() {
    let font = layout_font();
    let clip = Rect::new(24.0, 94.0, 3.0, 4.0);
    let mut visible = Vec::new();
    font.for_each_glyph("A V", 10.25, 100.5, |glyph| {
        let destination = Rect::new(
            glyph.x,
            glyph.y,
            glyph.source.width as f32,
            glyph.source.height as f32,
        );
        if let Some(bounds) = destination.intersect(clip)
            && bounds.w > 0.0
            && bounds.h > 0.0
        {
            visible.push((glyph, bounds));
        }
    });
    // A is outside the clip and the space has no rectangle; both still retain
    // their advances. V's untrimmed placement matches the unclipped consumer.
    assert_eq!(visible, [(placement(0, V_SOURCE, 23.25, 92.5), clip)]);
    assert_eq!(
        placed(&font, "A V"),
        [
            placement(0, A_SOURCE, 11.25, 93.5),
            placement(0, V_SOURCE, 23.25, 92.5),
        ]
    );
}
