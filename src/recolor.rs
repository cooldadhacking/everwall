//! The recoloring core.
//!
//! For every pixel we find its nearest palette colors in Oklab space and blend
//! them with Gaussian weights (a radial-basis-function interpolation, the same
//! idea `lutgen` uses). Blending the few nearest anchors — rather than snapping
//! to a single one — is what keeps smooth gradients smooth instead of banding
//! into flat posterized blocks, while still pulling the whole image onto the
//! theme's palette.

use image::RgbaImage;
use rayon::prelude::*;

use crate::color::{Oklab, oklab_to_srgb, srgb_to_oklab};
use crate::palette::{Group, Palette};

/// Tunable knobs for a recolor pass.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// How many nearest palette colors participate in each pixel's blend.
    /// 1 = hard snap (posterized); higher = smoother transitions.
    pub nearest: usize,
    /// Gaussian falloff. Higher = weights drop off faster, so the single
    /// nearest color dominates (crisper, more posterized). Lower = softer,
    /// more washed blends across neighbors.
    pub shape: f32,
    /// Keep each pixel's original perceptual lightness, letting the palette
    /// drive only hue and chroma. Preserves fine detail and contrast.
    pub preserve_luminance: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            nearest: 8,
            shape: 64.0,
            preserve_luminance: false,
        }
    }
}

/// Recolor `img` in place onto `palette` using `opts`.
pub fn recolor(img: &mut RgbaImage, palette: &Palette, opts: &Options) {
    let nearest = opts.nearest.clamp(1, palette.colors.len());

    // Deref-coerce the image down to its raw RGBA sample buffer and walk it in
    // 4-byte (one pixel) chunks across all cores.
    let buf: &mut [u8] = img;
    buf.par_chunks_exact_mut(4).for_each(|px| {
        // Fully transparent pixels carry no visible color; leave them be.
        if px[3] == 0 {
            return;
        }
        let src = srgb_to_oklab([px[0], px[1], px[2]]);
        let out = map_color(src, palette, nearest, opts.shape, opts.preserve_luminance);
        px[0] = out[0];
        px[1] = out[1];
        px[2] = out[2];
    });
}

/// Map a single source color onto the palette.
fn map_color(
    src: Oklab,
    palette: &Palette,
    nearest: usize,
    shape: f32,
    preserve_luminance: bool,
) -> [u8; 3] {
    // Distance from the source to every palette color, but the background ramp
    // is admitted only once. The ramp is seven near-neutral tones separated
    // almost purely by lightness, so for any dark pixel it occupies the entire
    // neighbor set and the accents never get a vote -- greens came out mauve.
    // Keeping just its closest member preserves the ramp's role as a lightness
    // reference while leaving the remaining slots for chromatic anchors.
    let mut dists: Vec<(f32, usize)> = Vec::with_capacity(palette.colors.len());
    let mut best_ramp: Option<(f32, usize)> = None;
    for (i, c) in palette.colors.iter().enumerate() {
        let d2 = src.dist_sq(&c.oklab);
        match c.group {
            Group::Ramp => {
                if best_ramp.is_none_or(|(bd, _)| d2 < bd) {
                    best_ramp = Some((d2, i));
                }
            }
            Group::Accent => dists.push((d2, i)),
        }
    }
    if let Some(r) = best_ramp {
        dists.push(r);
    }

    // Partial sort: we only need the `nearest` closest of what survived.
    let nearest = nearest.min(dists.len());
    dists.select_nth_unstable_by(nearest - 1, |a, b| a.0.total_cmp(&b.0));
    let chosen = &dists[..nearest];

    // Gaussian weight per chosen anchor, then normalize.
    let mut wl = 0.0f32;
    let mut wa = 0.0f32;
    let mut wb = 0.0f32;
    let mut wsum = 0.0f32;
    for &(d2, i) in chosen {
        let w = (-shape * d2).exp();
        let c = palette.colors[i].oklab;
        wl += w * c.l;
        wa += w * c.a;
        wb += w * c.b;
        wsum += w;
    }

    // If every weight underflowed to ~0 (source absurdly far from the palette),
    // fall back to the single nearest color.
    if wsum <= f32::MIN_POSITIVE {
        let i = chosen
            .iter()
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|&(_, i)| i)
            .unwrap_or(0);
        let c = palette.colors[i].oklab;
        let l = if preserve_luminance { src.l } else { c.l };
        return oklab_to_srgb(Oklab { l, a: c.a, b: c.b });
    }

    let l = if preserve_luminance { src.l } else { wl / wsum };
    oklab_to_srgb(Oklab {
        l,
        a: wa / wsum,
        b: wb / wsum,
    })
}
