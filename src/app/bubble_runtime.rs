//! App-owned bubble effects. This state never enters a Board or profile save.
//! Source timing is projected onto active Board ticks with a separate FX RNG.

use super::Presentation;
use crate::{
    adventure::{AdventurePhase, AdventureSession, GameMode},
    bubbles::{
        BubbleBlend, BubbleConfig, BubbleManager, BubbleRect, BubbleRenderOp, BubbleSpriteSource,
    },
    sim::Event,
};
use macroquad::prelude::*;

pub(super) struct BubbleRuntime {
    manager: BubbleManager,
    fx_seed: u64,
    ambient_sound_requested: bool,
}

impl BubbleRuntime {
    pub(super) fn new(seed: u64) -> Self {
        Self {
            manager: BubbleManager::new(BubbleConfig {
                bubble_bounds: BubbleRect::new(0, 82, 640, 398),
                max_bubbles: 0,
                bubble_spawn_chance: 0,
                max_fish: 0,
                fish_spawn_chance: 0,
                ..BubbleConfig::default()
            }),
            fx_seed: seed ^ 0xa24b_aed4_963e_e407,
            ambient_sound_requested: false,
        }
    }

    pub(super) fn clear(&mut self) {
        self.manager.clear();
        self.ambient_sound_requested = false;
    }

    pub(super) fn reinitialize(&mut self, seed: u64) {
        *self = Self::new(seed);
    }

    /// Consume exactly one unpaused Board simulation tick. Root caller owns
    /// pause/scene gating. BubbleMgr::Update precedes all Board requests.
    pub(super) fn advance(&mut self, session: &AdventureSession, events: &[Event]) {
        let virtual_board = (session.mode == GameMode::VirtualTank
            && matches!(session.phase, AdventurePhase::VirtualTank))
        .then_some(session.virtual_board.as_ref())
        .flatten();
        let ordinary_board = match session.mode {
            GameMode::Sandbox => session.sandbox_board.as_ref(),
            GameMode::VirtualTank => None,
            _ => session.board.as_ref(),
        };
        if virtual_board.is_none() && ordinary_board.is_none() {
            return;
        }

        let manager = &mut self.manager;
        let seed = &mut self.fx_seed;
        let mut roll = |upper| fx_roll(seed, upper);
        manager.update(&mut roll);

        let device_shown =
            virtual_board.is_some() && session.virtual_tank.settings.bubbulator_shown;
        if let Some(board) = virtual_board
            && device_shown
            && matches!(board.bubbulator_cycle(), 13 | 33 | 53)
        {
            let (base_x, base_y) = bubbulator_origin(session.virtual_tank.backdrop);
            let count = roll(5) + 5;
            for _ in 0..count {
                // These draws happen even when cap30 refuses the particle.
                let x = base_x + roll(41) as i32 + 5;
                let y = base_y - roll(10) as i32 - 15;
                manager.spawn_bubble_capped(x, y, 30, &mut roll);
            }
        }

        if !device_shown && roll(1000) == 0 {
            self.ambient_sound_requested = true;
            let count = roll(3) + 2;
            for _ in 0..count {
                // Board::SpawnRandomBubble checks cap50 before its Seed
                // coordinate draws. We use the private FX stream for both.
                if manager.bubbles().len() >= 50 {
                    continue;
                }
                let x = roll(22) as i32 + 150;
                let y = roll(6) as i32 + 400;
                manager.spawn_bubble_capped(x, y, 50, &mut roll);
            }
        }

        for event in events {
            match event {
                Event::GrubberBubble { x, y, .. } | Event::GekkoBubble { x, y, .. } => {
                    manager.spawn_bubble_capped(*x, *y, 30, &mut roll);
                }
                Event::WadsworthBubbles { positions, .. } => {
                    for &(x, y) in positions {
                        manager.spawn_bubble_capped(x, y, 30, &mut roll);
                    }
                }
                _ => {}
            }
        }
    }

    /// Board's ambient sound request occurs even when all bubble attempts
    /// meet cap50. The existing virtual Bubbulator event owns its own sound.
    pub(super) fn take_ambient_sound_request(&mut self) -> bool {
        std::mem::take(&mut self.ambient_sound_requested)
    }

    /// Render at the caller's Board overlay depth. Clip rectangles are
    /// applied by source/destination cropping, so caller scissor is untouched.
    /// Caller enters with default material; additive operations reset it.
    pub(super) fn draw(&self, presentation: &Presentation) {
        for op in self.manager.render_ops() {
            draw_op(presentation, op);
        }
    }
}

fn fx_roll(seed: &mut u64, upper: u64) -> u64 {
    debug_assert!(upper > 0);
    let mut next = *seed;
    if next == 0 {
        next = 0x9e37_79b9_7f4a_7c15;
    }
    next ^= next >> 12;
    next ^= next << 25;
    next ^= next >> 27;
    *seed = next;
    next.wrapping_mul(0x2545_f491_4f6c_dd1d) % upper
}

fn bubbulator_origin(backdrop: u8) -> (i32, i32) {
    match backdrop {
        0 => (470, 300),
        1 => (550, 315),
        2 => (10, 325),
        3 => (540, 315),
        _ => (360, 315),
    }
}

fn draw_op(presentation: &Presentation, op: BubbleRenderOp) {
    let id = op.sheet.resource_key();
    let Some(texture) = presentation.images.get(id) else {
        return;
    };
    let source = match op.source {
        BubbleSpriteSource::Cel(column) => {
            let Some(&(cols, rows)) = presentation.image_cells.get(id) else {
                return;
            };
            if cols == 0 || rows == 0 || u32::from(column) >= cols {
                return;
            }
            let width = texture.width() / cols as f32;
            let height = texture.height() / rows as f32;
            Rect::new(column as f32 * width, 0.0, width, height)
        }
        BubbleSpriteSource::Rect(rect) => Rect::new(
            rect.x as f32,
            rect.y as f32,
            rect.width as f32,
            rect.height as f32,
        ),
    };
    let Some((x, y, source)) = crop_to_clip(op.x, op.y, source, op.clip, op.mirror_x) else {
        return;
    };
    let tint = op.tint.map_or(WHITE, |color| {
        Color::from_rgba(color.red, color.green, color.blue, 255)
    });
    if op.blend == BubbleBlend::Additive {
        gl_use_material(&presentation.additive);
    }
    draw_texture_ex(
        texture,
        x,
        y,
        tint,
        DrawTextureParams {
            source: Some(source),
            dest_size: Some(vec2(source.w, source.h)),
            flip_x: op.mirror_x,
            ..Default::default()
        },
    );
    if op.blend == BubbleBlend::Additive {
        gl_use_default_material();
    }
}

fn crop_to_clip(
    x: f32,
    y: f32,
    mut source: Rect,
    clip: Option<BubbleRect>,
    mirror_x: bool,
) -> Option<(f32, f32, Rect)> {
    let Some(clip) = clip else {
        return Some((x, y, source));
    };
    let left = x.max(clip.x as f32);
    let top = y.max(clip.y as f32);
    let right = (x + source.w).min((clip.x + clip.width) as f32);
    let bottom = (y + source.h).min((clip.y + clip.height) as f32);
    if right <= left || bottom <= top {
        return None;
    }
    source.x += if mirror_x {
        x + source.w - right
    } else {
        left - x
    };
    source.y += top - y;
    source.w = right - left;
    source.h = bottom - top;
    Some((left, top, source))
}
