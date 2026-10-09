use std::{env, fs, path::Path};
use turbofish_openmpt::{Error, Module};

fn owned_module(game_root: &Path, name: &str) -> Module {
    let bytes = fs::read(game_root.join("music").join(name)).expect("read owned music");
    Module::from_bytes(&bytes).expect("decode owned music")
}

fn owned_root() -> std::path::PathBuf {
    env::var_os("OWNED_GAME_DIR")
        .map(Into::into)
        .expect("set OWNED_GAME_DIR to an owned Insaniquarium installation")
}

#[test]
fn invalid_module_bytes_return_errors() {
    assert!(matches!(
        Module::from_bytes(&[]),
        Err(Error::InvalidInput(_))
    ));
    let error = Module::from_bytes(b"not a module")
        .err()
        .expect("invalid bytes must fail");
    assert!(matches!(error, Error::Native { message, .. } if !message.is_empty()));
}

#[test]
#[ignore = "requires OWNED_GAME_DIR; Test-Runtime.ps1 -GameDirectory runs this"]
fn owned_module_rejects_invalid_buffers_and_bounds_writes() {
    let mut module = owned_module(&owned_root(), "Lullaby.mo3");
    assert!(matches!(
        module.render_stereo(7_999, &mut [0.0; 2]),
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        module.render_stereo(48_000, &mut [0.0; 3]),
        Err(Error::InvalidInput(_))
    ));
    assert!(matches!(
        module.render_stereo(48_000, &mut vec![0.0; 32_770]),
        Err(Error::InvalidInput(_))
    ));
    assert_eq!(module.render_stereo(48_000, &mut []).unwrap(), 0);
    assert!(matches!(
        module.set_repeat_count(-2),
        Err(Error::InvalidInput(_))
    ));
    let past_end = module.order_count().unwrap();
    assert!(matches!(
        module.seek(past_end, 0),
        Err(Error::InvalidInput(_))
    ));
    assert!(module.seek(0, 99_999).is_err());
    module.seek(0, 0).unwrap();
    module.set_repeat_count(-1).unwrap();
    module.set_repeat_count(0).unwrap();
    let mut backing = vec![f32::NAN; 1_024];
    assert_eq!(module.render_stereo(48_000, &mut backing[..8]).unwrap(), 4);
    assert!(backing[8..].iter().all(|sample| sample.is_nan()));
}

#[test]
#[ignore = "requires OWNED_GAME_DIR; Test-Runtime.ps1 -GameDirectory runs this"]
fn owned_orders_seek_and_render_finite_nonzero_pcm() {
    let game_root = owned_root();
    for (filename, orders) in [
        ("Insaniq2.mo3", &[0, 12, 25, 37][..]),
        ("Alien.mo3", &[0, 1, 3][..]),
        ("Lullaby.mo3", &[0][..]),
    ] {
        let mut module = owned_module(&game_root, filename);
        let count = module.order_count().unwrap();
        let mut pcm = vec![0.0; 16_384 * 2];
        for &order in orders {
            assert!(order < count, "{filename} order {order} out of range");
            module.seek(order, 0).unwrap();
            assert_eq!(module.current_order().unwrap(), order);
            let mut nonzero = 0;
            // The owned Lullaby intro has >100,000 silent frames.
            for _ in 0..30 {
                let frames = module.render_stereo(48_000, &mut pcm).unwrap();
                nonzero += pcm[..frames * 2]
                    .iter()
                    .filter(|sample| **sample != 0.0)
                    .count();
                if nonzero > 0 {
                    break;
                }
            }
            assert!(nonzero > 0, "{filename} order {order} remained silent");
        }
    }
}
