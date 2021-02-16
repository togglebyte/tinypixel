use std::mem::swap;

use crate::{Pixel, PixelBuffer, ScreenPos, ScreenSize};

/// Represents a drawable area on screen.
pub struct Viewport {
    /// The viewport's position on screen.
    /// Where 0.0,0 is the top left corner
    pub position: ScreenPos,

    /// The size of the viewport. Should probably match the size of the camera
    /// that is used with this viewport.
    pub size: ScreenSize,
    pub new_buf: PixelBuffer,
    old_buf: PixelBuffer,
    scale_factor: u32,
}

impl Viewport {
    /// Create a new viewport with a given screen position.
    pub fn new(position: ScreenPos, size: ScreenSize) -> Self {
        Self {
            position,
            size,
            new_buf: PixelBuffer::empty((size.width * size.height) as usize),
            old_buf: PixelBuffer::empty((size.width * size.height) as usize),
            scale_factor: 1,
        }
    }

    /// Resize the viewport.
    /// Remember to clear the renderer or residual
    /// characters might remain.
    pub fn resize(&mut self, new_size: ScreenSize) {
        self.size = ScreenSize::new(new_size.width, new_size.height);
        self.new_buf = PixelBuffer::empty((new_size.width * new_size.height) as usize);
        self.old_buf = PixelBuffer::empty((new_size.width * new_size.height) as usize);
    }

    /// Draw the pixels onto the renderable surface layers.
    /// This is offset by the camera and the viewport.
    pub fn draw_pixels(&mut self, pixels: Vec<(Pixel, ScreenPos)>) {
        pixels.iter().for_each(|(pixel, pos)| {
            self.draw_pixel(*pixel, *pos);
        });
    }

    /// Draw a single pixel onto the rendereable surface layers.
    /// This is called from `draw_pixels` for each pixel.
    pub fn draw_pixel(&mut self, pixel: Pixel, pos: ScreenPos) {
        if self.in_view(pos) {
            for x in 0..self.scale_factor {
                for y in 0..self.scale_factor {
                    let index = self.size.width * (pos.y + y) + pos.x + x;
                    self.new_buf.set_pixel(index as usize, pixel);
                }
            }
        }
    }

    /// Fill the entire viewport with one colour
    pub fn fill(&mut self, pixel: Pixel) {
        self.new_buf.inner.iter_mut().for_each(|p| *p = pixel);
    }

    /// Set the scale factor
    pub fn scale(&mut self, scale_factor: u32) {
        self.scale_factor = scale_factor;
    }

    fn in_view(&self, pos: ScreenPos) -> bool {
        pos.x < self.size.width && pos.y < self.size.height
    }

    fn offset(&self, pos: ScreenPos) -> ScreenPos {
        ScreenPos::new(pos.x + self.position.x, pos.y + self.position.y)
    }

    fn index_to_coords(&self, index: usize) -> ScreenPos {
        let x = index as u32 % self.size.width;
        let y = index as u32 / self.size.width;

        ScreenPos::new(x, y)
    }

    pub(crate) fn pixels(&mut self) -> Vec<(Pixel, ScreenPos)> {
        let mut pixels = Vec::new();

        for (new, old) in self
            .new_buf
            .inner
            .iter()
            .enumerate()
            .zip(&self.old_buf.inner)
        {
            match (new, old) {
                ((_, new), old) if new == old => {}
                ((index, Pixel { a: 0, .. }), old_pixel) => {
                    if old_pixel.a > 0 {
                        let pos = self.offset(self.index_to_coords(index));
                        pixels.push((Pixel::zero(), pos));
                    }
                }
                ((index, pixel), _) => {
                    let pos = self.offset(self.index_to_coords(index));
                    pixels.push((*pixel, pos));
                }
            }
        }

        swap(&mut self.new_buf, &mut self.old_buf);
        self.new_buf.zero();

        pixels
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;

    fn camera(viewport: &Viewport) -> Camera<camera::NoLimit> {
        let pos = WorldPos::new(30.0, 30.0);
        Camera::from_viewport(pos, viewport)
    }

    fn viewport() -> Viewport {
        let pos = ScreenPos::new(2, 2);
        let size = ScreenSize::new(6, 6);
        Viewport::new(pos, size)
    }

    #[test]
    fn draw_corners() {
        let mut view = viewport();
        let cam = camera(&view);

        let min_x = cam.bounding_box.min_x();
        let max_x = cam.bounding_box.max_x();
        let min_y = cam.bounding_box.min_y();
        let max_y = cam.bounding_box.max_y();

        let a = WorldPos::new(min_x, min_y);
        let b = WorldPos::new(max_x - 1.0, min_y);
        let c = WorldPos::new(min_x, max_y - 1.0);
        let d = WorldPos::new(max_x - 1.0, max_y - 1.0);

        let positions = vec![a, b, c, d];
        let glyphs = vec!['A', 'B', 'C', 'D'];
        let pixels = positions
            .into_iter()
            .zip(glyphs)
            .map(|(p, g)| Pixel::new(g, cam.to_screen(p), None, None))
            .collect::<Vec<_>>();

        view.draw_pixels(pixels);

        let a = Pixel::new('A', ScreenPos::new(2, 2), None, None);
        let b = Pixel::new('B', ScreenPos::new(7, 2), None, None);
        let c = Pixel::new('C', ScreenPos::new(2, 7), None, None);
        let d = Pixel::new('D', ScreenPos::new(7, 7), None, None);

        let drawn_pixels = view.pixels();

        assert_eq!(&drawn_pixels, &[a, b, c, d]);
    }
}
