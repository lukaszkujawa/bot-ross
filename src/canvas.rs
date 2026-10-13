use crate::color::Premultiplied;
use crate::image::Image;
use crate::layer::Layer;

pub struct Canvas {
    pub width: usize,
    pub height: usize,
    background: Premultiplied,
    layers: Vec<Layer>,
}

impl Canvas {
    pub fn new(width: usize, height: usize, background: Premultiplied) -> Self {
        Self { width, height, background, layers: Vec::new() }
    }

    pub fn add_layer(&mut self) -> &mut Layer {
        self.layers.push(Layer::new(self.width, self.height));
        self.layers.last_mut().expect("layer was just pushed")
    }

    pub fn composite(&self) -> Vec<Premultiplied> {
        let mut composite = vec![self.background; self.width * self.height];
        for layer in &self.layers {
            for (below, above) in composite.iter_mut().zip(layer.pixels()) {
                *below = above.over(*below);
            }
        }
        composite
    }

    pub fn render(&self) -> Image {
        Image { width: self.width, height: self.height, argb: self.composite().into_iter().map(Premultiplied::to_argb).collect() }
    }
}
