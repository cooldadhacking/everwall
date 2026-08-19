//! sRGB <-> Oklab conversion.
//!
//! Oklab (Björn Ottosson, 2020) is a perceptual color space where Euclidean
//! distance tracks perceived color difference far better than raw RGB. We map
//! every pixel and every palette color into Oklab, do all nearest-color and
//! blending math there, then convert back to sRGB for output.

/// A color in the Oklab space: `l` = perceptual lightness, `a`/`b` = chroma axes.
#[derive(Clone, Copy, Debug)]
pub struct Oklab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

impl Oklab {
    /// Squared Euclidean distance — cheaper than the true distance and monotonic,
    /// so it orders and weights colors identically.
    #[inline]
    pub fn dist_sq(&self, other: &Oklab) -> f32 {
        let dl = self.l - other.l;
        let da = self.a - other.a;
        let db = self.b - other.b;
        dl * dl + da * da + db * db
    }
}

/// sRGB gamma decode: 8-bit gamma-encoded channel -> linear [0, 1].
#[inline]
fn srgb_to_linear(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// sRGB gamma encode: linear [0, 1] -> 8-bit gamma-encoded channel.
#[inline]
fn linear_to_srgb(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let v = if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    };
    (v * 255.0).round().clamp(0.0, 255.0) as u8
}

/// Convert a gamma-encoded sRGB triple into Oklab.
pub fn srgb_to_oklab(rgb: [u8; 3]) -> Oklab {
    let r = srgb_to_linear(rgb[0]);
    let g = srgb_to_linear(rgb[1]);
    let b = srgb_to_linear(rgb[2]);

    let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
    let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
    let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    Oklab {
        l: 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        a: 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        b: 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
    }
}

/// Convert an Oklab color back into a gamma-encoded sRGB triple (clamped to gamut).
pub fn oklab_to_srgb(c: Oklab) -> [u8; 3] {
    let l_ = c.l + 0.3963377774 * c.a + 0.2158037573 * c.b;
    let m_ = c.l - 0.1055613458 * c.a - 0.0638541728 * c.b;
    let s_ = c.l - 0.0894841775 * c.a - 1.2914855480 * c.b;

    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.6413753702 * s;
    let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

    [linear_to_srgb(r), linear_to_srgb(g), linear_to_srgb(b)]
}

/// Parse a `#RRGGBB` (or `RRGGBB`) hex string into an sRGB triple.
pub fn parse_hex(hex: &str) -> Option<[u8; 3]> {
    let h = hex.strip_prefix('#').unwrap_or(hex);
    if h.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&h[0..2], 16).ok()?;
    let g = u8::from_str_radix(&h[2..4], 16).ok()?;
    let b = u8::from_str_radix(&h[4..6], 16).ok()?;
    Some([r, g, b])
}
