use crate::color::OPAQUE;
use crate::painting::selection::{Ellipse, Frame, Mode, Prepared, Selection};
use crate::reference::Reference;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct Region {
    pub centre: Option<[f32; 2]>,
    pub radius: Option<f32>,
    pub aspect: f32,
    pub weight: f32,
    pub outside: f32,
    pub feather: f32,
    pub relative: bool,
    pub select: Vec<Selection>,
}

impl Default for Region {
    fn default() -> Self {
        Self { centre: None, radius: None, aspect: 1.0, weight: 3.0, outside: 1.0, feather: 0.5, relative: false, select: Vec::new() }
    }
}

impl Region {
    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        if self.centre.is_some() != self.radius.is_some() {
            problems.push("region centre and radius go together".into());
        }
        if self.centre.is_none() && self.select.is_empty() {
            problems.push("region needs a centre and radius or a select list".into());
        }
        if self.aspect <= 0.0 {
            problems.push("region aspect must be greater than 0".into());
        }
        if self.radius.is_some_and(|radius| radius <= 0.0) {
            problems.push("region radius must be greater than 0".into());
        }
        if self.weight < 0.0 || self.outside < 0.0 || self.feather < 0.0 {
            problems.push("region weight, outside and feather must not be negative".into());
        }
        for (index, selection) in self.select.iter().enumerate() {
            problems.extend(selection.validate(&format!("region select {index}")));
        }
        problems
    }

    pub fn mask(&self, reference: &Reference) -> Vec<f32> {
        self.coverage(reference).into_iter().map(|coverage| self.weigh(coverage)).collect()
    }

    pub fn weigh(&self, coverage: f32) -> f32 {
        self.outside + (self.weight - self.outside) * coverage
    }

    pub fn coverage(&self, reference: &Reference) -> Vec<f32> {
        let (width, height) = (reference.width as f32, reference.height as f32);
        let frame = if self.relative { Frame { x: width, y: height, size: width.min(height) } } else { Frame { x: 1.0, y: 1.0, size: 1.0 } };
        let prepared: Vec<Prepared> = self.selections().iter().map(|selection| selection.prepare(frame, self.feather)).collect();
        reference
            .pixels()
            .iter()
            .enumerate()
            .map(|(index, pixel)| {
                let (x, y) = ((index % reference.width) as f32 + 0.5, (index / reference.width) as f32 + 0.5);
                let rgb = pixel.map(|channel| channel as f32 / OPAQUE as f32);
                prepared.iter().fold(0.0, |covered: f32, selection| match selection.mode {
                    Mode::Add => covered.max(selection.coverage(x, y, rgb)),
                    Mode::Subtract => covered.min(1.0 - selection.coverage(x, y, rgb)),
                    Mode::Intersect => covered.min(selection.coverage(x, y, rgb)),
                })
            })
            .collect()
    }

    fn selections(&self) -> Vec<Selection> {
        let shorthand = self.centre.zip(self.radius).map(|(centre, radius)| Selection { ellipse: Some(Ellipse { centre, radius, aspect: self.aspect }), ..Selection::default() });
        shorthand.into_iter().chain(self.select.iter().cloned()).collect()
    }
}
