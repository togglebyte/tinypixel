use std::ops::{Deref, DerefMut};


// -----------------------------------------------------------------------------
//     - Pixel -
// -----------------------------------------------------------------------------
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Pixel {
    pub fn zero() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}

unsafe impl bytemuck::Pod for Pixel {}
unsafe impl bytemuck::Zeroable for Pixel {}

// -----------------------------------------------------------------------------
//     - Pixel buffer -
// -----------------------------------------------------------------------------
pub struct PixelBuffer {
    pub(crate) inner: Vec<Pixel>,
}

impl PixelBuffer {
    pub fn empty(cap: usize) -> Self {
        Self {
            inner: (0..cap).map(|_| Pixel::zero()).collect(),
        }
    }

    pub fn new(cap: usize, pixel: Pixel) -> Self {
        Self {
            inner: (0..cap).map(|_| pixel).collect(),
        }
    }

    pub fn set_pixel(&mut self, index: usize, pixel: Pixel) {
        self.inner[index] = pixel;
    }

    pub fn zero(&mut self) {
        bytemuck::cast_slice_mut(&mut self.inner).fill(0);
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

