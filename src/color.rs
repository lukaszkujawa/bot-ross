use serde::{Deserialize, Serialize};

pub const OPAQUE: u16 = 1 << 15;
const HUE_SHARE: f32 = 0.15;

pub fn parse_hex(hex: &str) -> Result<[f32; 3], String> {
    let digits = hex.strip_prefix('#').filter(|digits| digits.len() == 6).ok_or_else(|| format!("colour must be #rrggbb, got {hex}"))?;
    let channel = |index: usize| digits.get(index..index + 2).and_then(|pair| u8::from_str_radix(pair, 16).ok()).map(|v| v as f32 / 255.0).ok_or_else(|| format!("colour must be #rrggbb, got {hex}"));
    Ok([channel(0)?, channel(2)?, channel(4)?])
}

pub fn hex([r, g, b]: [f32; 3]) -> String {
    let byte = |channel: f32| (channel.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", byte(r), byte(g), byte(b))
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct Swatch(pub Hsv);

impl TryFrom<String> for Swatch {
    type Error = String;

    fn try_from(hex: String) -> Result<Self, String> {
        parse_hex(&hex).map(|rgb| Self(Hsv::from_rgb(rgb)))
    }
}

impl From<Swatch> for String {
    fn from(swatch: Swatch) -> String {
        hex(swatch.0.to_rgb())
    }
}

pub fn pick_swatch(palette: &[Swatch], pick: &mut impl FnMut([f32; 2]) -> f32) -> Option<Hsv> {
    let index = ((pick([0.0, 1.0]) * palette.len() as f32) as usize).min(palette.len().checked_sub(1)?);
    Some(palette[index].0)
}

pub fn luminance([r, g, b]: [f32; 3]) -> f32 {
    r * 0.3 + g * 0.5 + b * 0.2
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsv {
    pub hue: f32,
    pub saturation: f32,
    pub value: f32,
}

impl Hsv {
    pub fn shaded(self, delta: f32) -> Self {
        Self { value: (self.value + delta).clamp(0.0, 1.0), ..self }
    }

    pub fn saturated(self, saturation: f32) -> Self {
        Self { saturation: (self.saturation * saturation).clamp(0.0, 1.0), ..self }
    }

    pub fn dimmed(self, factor: f32) -> Self {
        Self { value: (self.value * factor).clamp(0.0, 1.0), ..self }
    }

    pub fn contrasted(self, contrast: f32) -> Self {
        Self { value: (0.5 + (self.value - 0.5) * contrast).clamp(0.0, 1.0), ..self }
    }

    pub fn jittered(self, amount: f32, pick: &mut impl FnMut([f32; 2]) -> f32) -> Self {
        let range = [-amount, amount];
        Self {
            hue: (self.hue + pick(range) * HUE_SHARE).rem_euclid(1.0),
            saturation: (self.saturation + pick(range)).clamp(0.0, 1.0),
            value: (self.value + pick(range)).clamp(0.0, 1.0),
        }
    }

    pub fn from_rgb([r, g, b]: [f32; 3]) -> Self {
        let (max, min) = (r.max(g).max(b), r.min(g).min(b));
        let chroma = max - min;
        let hue = if chroma <= 0.0 {
            0.0
        } else if max == r {
            ((g - b) / chroma).rem_euclid(6.0)
        } else if max == g {
            (b - r) / chroma + 2.0
        } else {
            (r - g) / chroma + 4.0
        };
        Self { hue: hue / 6.0, saturation: if max <= 0.0 { 0.0 } else { chroma / max }, value: max }
    }

    pub fn to_rgb(self) -> [f32; 3] {
        let sector = self.hue.rem_euclid(1.0) * 6.0;
        let chroma = self.value * self.saturation;
        let second = chroma * (1.0 - (sector % 2.0 - 1.0).abs());
        let base = self.value - chroma;
        let (r, g, b) = match sector as u8 {
            0 => (chroma, second, 0.0),
            1 => (second, chroma, 0.0),
            2 => (0.0, chroma, second),
            3 => (0.0, second, chroma),
            4 => (second, 0.0, chroma),
            _ => (chroma, 0.0, second),
        };
        [r + base, g + base, b + base]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Premultiplied {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
    pub alpha: u16,
}

impl Premultiplied {
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);
    pub const WHITE: Self = Self::new(OPAQUE, OPAQUE, OPAQUE, OPAQUE);

    pub const fn new(red: u16, green: u16, blue: u16, alpha: u16) -> Self {
        Self { red, green, blue, alpha }
    }

    pub fn from_rgb(rgb: [f32; 3], alpha: f32) -> Self {
        let alpha = alpha.clamp(0.0, 1.0);
        let channel = |value: f32| (value.clamp(0.0, 1.0) * alpha * OPAQUE as f32).round() as u16;
        Self::new(channel(rgb[0]), channel(rgb[1]), channel(rgb[2]), (alpha * OPAQUE as f32).round() as u16)
    }

    pub fn to_channels(self) -> [u16; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }

    pub fn to_rgb(self) -> [f32; 3] {
        let unpremultiply = |channel: u16| if self.alpha == 0 { 0.0 } else { channel as f32 / self.alpha as f32 };
        [unpremultiply(self.red), unpremultiply(self.green), unpremultiply(self.blue)]
    }

    pub fn over(self, background: Self) -> Self {
        let keep = |channel: u16| (channel as u32 * (OPAQUE - self.alpha) as u32 / OPAQUE as u32) as u16;
        Self::new(
            self.red + keep(background.red),
            self.green + keep(background.green),
            self.blue + keep(background.blue),
            self.alpha + keep(background.alpha),
        )
    }

    pub fn multiply(self, below: Self) -> Self {
        let alpha = self.over(below).alpha;
        let through = |above: u16, under: u16| {
            let transmitted = (OPAQUE - self.alpha + above) as u32 * (OPAQUE - below.alpha + under) as u32 / OPAQUE as u32;
            transmitted.saturating_sub((OPAQUE - alpha) as u32) as u16
        };
        Self::new(through(self.red, below.red), through(self.green, below.green), through(self.blue, below.blue), alpha)
    }

    pub fn mix(self, other: Self, weight: f32) -> Self {
        let lerp = |a: u16, b: u16| (a as f32 + (b as f32 - a as f32) * weight) as u16;
        Self::new(lerp(self.red, other.red), lerp(self.green, other.green), lerp(self.blue, other.blue), lerp(self.alpha, other.alpha))
    }

    pub fn to_argb(self) -> u32 {
        let byte = |channel: u16| (channel as u32 * 255 / OPAQUE as u32).min(255);
        (byte(self.red) << 16) | (byte(self.green) << 8) | byte(self.blue)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Blend {
    #[default]
    Normal,
    Multiply,
}

impl Blend {
    pub fn apply(self, above: Premultiplied, below: Premultiplied) -> Premultiplied {
        match self {
            Self::Normal => above.over(below),
            Self::Multiply => above.multiply(below),
        }
    }
}
