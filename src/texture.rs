use std::mem::size_of;

use crate::{PixelBuffer, Pixel};

pub struct Texture {
    pub inner: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: wgpu::Extent3d,
}

impl Texture {

    pub fn empty(device: &wgpu::Device, queue: &wgpu::Queue, size: crate::ScreenSize) -> Texture {
        let pixels = PixelBuffer::new((size.width * size.height) as usize, Pixel::zero());
        Self::new(&pixels, device, queue, size)
    }

    pub fn new(pixels: &[u8], device: &wgpu::Device, queue: &wgpu::Queue, size: crate::ScreenSize) -> Texture {

        let size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture label"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });


        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &pixels,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: size_of::<Pixel>() as u32 * size.width,
                rows_per_image: size.height,
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default()); 

        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }
        );

        Self { inner: texture, view, sampler, size }
    }
}
