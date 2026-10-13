pub const OPAQUE: u16 = 1 << 15;

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

    pub fn contrasted(self, contrast: f32) -> Self {
        Self { value: (0.5 + (self.value - 0.5) * contrast).clamp(0.0, 1.0), ..self }
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

    pub fn mix(self, other: Self, weight: f32) -> Self {
        let lerp = |a: u16, b: u16| (a as f32 + (b as f32 - a as f32) * weight) as u16;
        Self::new(lerp(self.red, other.red), lerp(self.green, other.green), lerp(self.blue, other.blue), lerp(self.alpha, other.alpha))
    }

    pub fn to_argb(self) -> u32 {
        let byte = |channel: u16| (channel as u32 * 255 / OPAQUE as u32).min(255);
        (byte(self.red) << 16) | (byte(self.green) << 8) | byte(self.blue)
    }
}
