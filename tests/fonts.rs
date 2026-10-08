use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use turbofish_deluxe::font::{BitmapFont, GlyphRect};

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
