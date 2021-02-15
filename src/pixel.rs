use std::ops::{Deref, DerefMut};


// -----------------------------------------------------------------------------
//     - Pixel -
// -----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel {
    fn black() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }
    }

    fn blue() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 100,
            a: 255,
        }
    }
}

unsafe impl bytemuck::Pod for Pixel {}
unsafe impl bytemuck::Zeroable for Pixel {}

// -----------------------------------------------------------------------------
//     - Pixel buffer -
// -----------------------------------------------------------------------------
pub struct PixelBuffer {
    inner: Vec<Pixel>,
}

impl PixelBuffer {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            inner: (0..cap).map(|_| Pixel::blue()).collect(),
        }
    }

    pub fn set_pixel(&mut self, index: usize, pixel: Pixel) {
        self.inner[index] = pixel;
    }
}

impl Deref for PixelBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        bytemuck::cast_slice(&self.inner)
    }
}

impl DerefMut for PixelBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        bytemuck::cast_slice_mut(&mut self.inner)
    }
}

