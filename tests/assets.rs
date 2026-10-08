use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use turbofish_deluxe::assets::{GameAssets, SoundData};

struct Fixture(PathBuf);

impl Fixture {
    fn new(manifest: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("turbofish-assets-{}-{nonce}", std::process::id()));
        fs::create_dir_all(path.join("properties")).expect("fixture properties");
        fs::create_dir_all(path.join("images")).expect("fixture images");
        fs::create_dir_all(path.join("sounds")).expect("fixture sounds");
        fs::write(path.join("properties/resources.xml"), manifest).expect("fixture manifest");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn image(&self, name: &str, width: u32, height: u32, rgb: [u8; 3]) {
        let image = image::RgbImage::from_pixel(width, height, image::Rgb(rgb));
        image
            .save(self.0.join("images").join(name))
            .expect("fixture image");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove fixture");
    }
}

const MANIFEST: &str = "<ResourceManifest><Resources id=Tank><SetDefaults path=images idprefix=IMAGE_/><Image id=FISH path=fish cols=2 rows=1 ddsurface/><Image id=MASKONLY path=maskonly/><Image id=GRID path=grid cols=2 alphagrid=gridmask/><Image id=EXPLICIT path=explicit alphaimage=shared/><SetDefaults path=sounds idprefix=SOUND_/><Sound id=CLICK path=click/></Resources></ResourceManifest>";

#[test]
fn parses_stock_attribute_forms_and_replaces_alpha_with_mask_blue() {
    let fixture = Fixture::new(MANIFEST);
    fixture.image("fish.png", 2, 1, [12, 34, 56]);
    fixture.image("_fish.png", 2, 1, [255, 0, 80]);
    let assets = GameAssets::open(fixture.path()).expect("manifest");
    let entry = assets.image_entry("IMAGE_FISH").expect("image id");
    assert_eq!((entry.cols, entry.rows), (2, 1));
    let decoded = assets.load_image("IMAGE_FISH").expect("RGBA");
    assert_eq!(
        (decoded.width, decoded.height, decoded.cols, decoded.rows),
        (2, 1, 2, 1)
    );
    assert_eq!(&decoded.pixels[..4], &[12, 34, 56, 80]);
}

#[test]
fn alpha_only_image_is_white_rgb_with_mask_blue_alpha() {
    let fixture = Fixture::new(MANIFEST);
    fixture.image("_maskonly.png", 1, 1, [0, 0, 201]);
    let decoded = GameAssets::open(fixture.path())
        .unwrap()
        .load_image("IMAGE_MASKONLY")
        .unwrap();
    assert_eq!(decoded.pixels, vec![255, 255, 255, 201]);
}

#[test]
fn explicit_alpha_image_covers_full_sheet_and_alpha_grid_repeats_per_cell() {
    let fixture = Fixture::new(MANIFEST);
    fixture.image("grid.png", 2, 1, [4, 5, 6]);
    fixture.image("gridmask.png", 1, 1, [0, 0, 77]);
    fixture.image("explicit.png", 2, 1, [7, 8, 9]);
    fixture.image("shared.png", 2, 1, [0, 0, 88]);
    let assets = GameAssets::open(fixture.path()).unwrap();
    let grid = assets.load_image("IMAGE_GRID").unwrap();
    assert_eq!(&grid.pixels[..8], &[4, 5, 6, 77, 4, 5, 6, 77]);
    let explicit = assets.load_image("IMAGE_EXPLICIT").unwrap();
    assert_eq!(&explicit.pixels[..8], &[7, 8, 9, 88, 7, 8, 9, 88]);
}

#[test]
fn ignores_mismatched_auto_mask_but_rejects_declared_mismatch_and_escape() {
    let fixture = Fixture::new(MANIFEST);
    fixture.image("fish.png", 2, 1, [1, 2, 3]);
    fixture.image("_fish.png", 1, 1, [0, 0, 255]);
    let assets = GameAssets::open(fixture.path()).unwrap();
    assert_eq!(
        &assets.load_image("IMAGE_FISH").unwrap().pixels[..4],
        &[1, 2, 3, 255]
    );
    fixture.image("grid.png", 2, 1, [1, 2, 3]);
    fixture.image("gridmask.png", 2, 1, [0, 0, 255]);
    assert!(
        assets
            .load_image("IMAGE_GRID")
            .unwrap_err()
            .to_string()
            .contains("mask dimensions")
    );
    assert!(
        assets
            .load_image("../outside")
            .unwrap_err()
            .to_string()
            .contains("unsafe")
    );
    let bad = Fixture::new(
        "<ResourceManifest><Resources><Image id=X path=../../outside/></Resources></ResourceManifest>",
    );
    assert!(
        GameAssets::open(bad.path())
            .unwrap_err()
            .to_string()
            .contains("unsafe")
    );
    let malformed = Fixture::new(
        "<ResourceManifest><Resources><Image id=X path=fish cols=0/></Resources></ResourceManifest>",
    );
    assert!(
        GameAssets::open(malformed.path())
            .unwrap_err()
            .to_string()
            .contains("invalid cols")
    );
}

#[test]
fn decodes_known_mulaw_endpoints_and_rejects_truncated_payload() {
    let fixture = Fixture::new(MANIFEST);
    let mut au = Vec::new();
    au.extend_from_slice(b".snd");
    for number in [24_u32, 3, 1, 8012, 1] {
        au.extend_from_slice(&number.to_be_bytes());
    }
    au.extend_from_slice(&[0xff, 0x80, 0x00]);
    fs::write(fixture.path().join("sounds/click.au"), &au).unwrap();
    let assets = GameAssets::open(fixture.path()).unwrap();
    match assets.load_sound("SOUND_CLICK").unwrap() {
        SoundData::Pcm16 {
            sample_rate,
            samples,
        } => {
            assert_eq!(sample_rate, 8012);
            assert_eq!(samples, vec![0, 32124, -32124]);
        }
        other => panic!("unexpected sound {other:?}"),
    }
    au.truncate(25);
    fs::write(fixture.path().join("sounds/click.au"), au).unwrap();
    assert!(
        assets
            .load_sound("SOUND_CLICK")
            .unwrap_err()
            .to_string()
            .contains("truncated")
    );
}
