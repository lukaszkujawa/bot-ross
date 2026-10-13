use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Span {
    Fixed(f32),
    Range([f32; 2]),
}

impl Span {
    pub fn range(self) -> [f32; 2] {
        match self {
            Self::Fixed(value) => [value, value],
            Self::Range(range) => range,
        }
    }
}

impl From<f32> for Span {
    fn from(value: f32) -> Self {
        Self::Fixed(value)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(tag = "style", rename_all = "lowercase", deny_unknown_fields, bound(deserialize = "T: Deserialize<'de> + From<f32>"))]
pub enum Fill<T> {
    #[default]
    Solid,
    Hatch {
        spacing: T,
        #[serde(default = "one")]
        width: T,
        angle: Option<T>,
        #[serde(default)]
        cross: bool,
    },
    Dots {
        spacing: T,
        #[serde(default = "one")]
        radius: T,
    },
    Outline {
        #[serde(default = "one")]
        width: T,
    },
}

fn one<T: From<f32>>() -> T {
    T::from(1.0)
}

impl Fill<Span> {
    pub fn pick(&self, pick: &mut impl FnMut([f32; 2]) -> f32) -> Fill<f32> {
        let mut pick = |span: &Span| pick(span.range());
        match self {
            Self::Solid => Fill::Solid,
            Self::Hatch { spacing, width, angle, cross } => Fill::Hatch { spacing: pick(spacing), width: pick(width), angle: angle.as_ref().map(&mut pick), cross: *cross },
            Self::Dots { spacing, radius } => Fill::Dots { spacing: pick(spacing), radius: pick(radius) },
            Self::Outline { width } => Fill::Outline { width: pick(width) },
        }
    }

    pub fn validate(&self, label: &str) -> Vec<String> {
        let (positive, any) = match self {
            Self::Solid => (vec![], vec![]),
            Self::Hatch { spacing, width, angle, .. } => (vec![("spacing", *spacing), ("width", *width)], angle.map(|angle| ("angle", angle)).into_iter().collect()),
            Self::Dots { spacing, radius } => (vec![("spacing", *spacing), ("radius", *radius)], vec![]),
            Self::Outline { width } => (vec![("width", *width)], vec![]),
        };
        let mut problems = Vec::new();
        for (key, span) in positive.iter().chain(&any) {
            let [min, max] = span.range();
            if min > max {
                problems.push(format!("{label}: fill {key} must be a value or [min, max] with min <= max"));
            }
        }
        for (key, span) in &positive {
            if span.range()[0] <= 0.0 {
                problems.push(format!("{label}: fill {key} must be greater than 0"));
            }
        }
        problems
    }
}

impl Fill<f32> {
    pub fn coverage(&self, distance: f32, edge: f32, u: f32, v: f32) -> f32 {
        let inside = ((edge - distance) / (2.0 * edge)).clamp(0.0, 1.0);
        match self {
            Self::Solid => inside,
            Self::Outline { width } => ((width / 2.0 + edge - distance.abs()) / (2.0 * edge)).clamp(0.0, 1.0),
            Self::Hatch { spacing, width, cross, .. } => {
                let line = |t: f32| {
                    let offset = t.rem_euclid(*spacing);
                    (width / 2.0 + 0.5 - offset.min(spacing - offset)).clamp(0.0, 1.0)
                };
                inside * if *cross { line(u).max(line(v)) } else { line(v) }
            }
            Self::Dots { spacing, radius } => {
                let nearest = |t: f32| t - (t / spacing).round() * spacing;
                inside * (radius + 0.5 - nearest(u).hypot(nearest(v))).clamp(0.0, 1.0)
            }
        }
    }

    pub fn reach(&self) -> f32 {
        match self {
            Self::Outline { width } => width / 2.0,
            _ => 0.0,
        }
    }

    pub fn frame(&self, rotation: f32) -> f32 {
        match self {
            Self::Hatch { angle: Some(degrees), .. } => degrees.to_radians(),
            _ => rotation,
        }
    }
}
