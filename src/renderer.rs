use std::mem::size_of;

use futures::executor::block_on;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

use crate::{texture, Pixel, PixelBuffer, ScreenPos, ScreenSize, Viewport};

// -----------------------------------------------------------------------------
//     - Vertex-
// -----------------------------------------------------------------------------
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                },
            ],
        }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

// -----------------------------------------------------------------------------
//     - Square -
//     Drawing area
// -----------------------------------------------------------------------------
const VERTICES: &[Vertex] = &[
    // Top left 0
    Vertex {
        position: [-1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    // Top right 1
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    // Bottom left 2
    Vertex {
        position: [-1.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    // Bottom right 3
    Vertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
];

const INDICES: &[u16] = &[0, 2, 3, 0, 3, 1];

// -----------------------------------------------------------------------------
//     - Renderer -
// -----------------------------------------------------------------------------
pub struct Renderer {
    state: State,
    pixels: PixelBuffer,
}

impl Renderer {
    fn coords_to_index(&self, pos: ScreenPos) -> usize {
        (pos.x + pos.y * self.state.size.width) as usize
    }

    pub fn draw(&mut self, viewport: &mut Viewport) {
        let pixels = viewport.pixels();
        if pixels.len() > 0 {
            eprintln!("{:?}", pixels.len());
        }
        pixels.into_iter().for_each(|(pix, pos)| {
            let index = self.coords_to_index(pos);
            if index < self.pixels.inner.len() {
                self.pixels.inner[index] = pix;
            }
        });
    }

    pub fn render(&mut self) {
        self.state.queue.write_texture(
            wgpu::TextureCopyView {
                texture: &self.state.texture.inner,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &self.pixels,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: size_of::<Pixel>() as u32 * self.state.size.width,
                rows_per_image: self.state.size.height,
            },
            self.state.texture.size,
        );
        self.state.render();
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let cap = new_size.width * new_size.height;
        let pixels = PixelBuffer::new(cap as usize, Pixel::zero());
        self.pixels = pixels;
        self.state.resize(new_size);
    }

    pub fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let cap = size.width * size.height;
        let pixels = PixelBuffer::new(cap as usize, Pixel::zero());

        Self {
            state: block_on(State::new(window)),
            pixels,
        }
    }
}

fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    dimension: wgpu::TextureViewDimension::D2,
                    component_type: wgpu::TextureComponentType::Uint,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler { comparison: false },
                count: None,
            },
        ],
        label: Some("texture binding group layout"),
    })
}

fn bind_group(device: &wgpu::Device, texture: &texture::Texture) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout(&device),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            },
        ],
        label: Some("Texture bind group"),
    })
}

// -----------------------------------------------------------------------------
//     - State-
//     Maybe absolute nonsense:
//     Device -> [ Queue -> SwapChain -> RenderPipeline -> Surface ]
// -----------------------------------------------------------------------------
struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    diffuse_bind_group: wgpu::BindGroup,
    texture: texture::Texture,
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // -----------------------------------------------------------------------------
        //     - Texture -
        // -----------------------------------------------------------------------------
        let texture =
            texture::Texture::empty(&device, &queue, ScreenSize::new(size.width, size.height));

        let texture_bind_group_layout = bind_group_layout(&device);

        let diffuse_bind_group = bind_group(&device, &texture);

        // -----------------------------------------------------------------------------
        //     - Shader bits -
        // -----------------------------------------------------------------------------
        let vs_module = device.create_shader_module(wgpu::include_spirv!("shader.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"));

        // buffer business
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex buffer yaaaay"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index buffer because things aren't hard enough as they are"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        // -----------------------------------------------------------------------------
        //     - Pipeline -
        // -----------------------------------------------------------------------------
        let render_pipeline = create_pipeline(
            &device,
            &sc_desc,
            vs_module,
            fs_module,
            texture_bind_group_layout,
        );

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: INDICES.len() as u32,
            diffuse_bind_group,
            texture,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);

        let texture = texture::Texture::empty(
            &self.device,
            &self.queue,
            ScreenSize::new(new_size.width, new_size.height),
        );

        self.diffuse_bind_group = bind_group(&self.device, &texture);

        self.texture = texture;
    }

    fn render(&mut self) {
        let frame = self
            .swap_chain
            .get_current_frame()
            .expect("Timeout?")
            .output;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..));
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

// -----------------------------------------------------------------------------
//     - Create pipeline -
// -----------------------------------------------------------------------------
fn create_pipeline(
    device: &wgpu::Device,
    sc_desc: &wgpu::SwapChainDescriptor,
    vs_module: wgpu::ShaderModule,
    fs_module: wgpu::ShaderModule,
    texture_bind_group: wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render pipeline layout what does this even mean"),
        bind_group_layouts: &[&texture_bind_group],
        push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Pipeline omg pipeline (render okay)"),
        layout: Some(&render_pipeline_layout),
        vertex_stage: wgpu::ProgrammableStageDescriptor {
            module: &vs_module,
            entry_point: "main",
        },
        fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
            module: &fs_module,
            entry_point: "main",
        }),
        rasterization_state: Some(wgpu::RasterizationStateDescriptor {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: wgpu::CullMode::Back,
            depth_bias: 0,
            depth_bias_slope_scale: 0.0,
            depth_bias_clamp: 0.0,
            clamp_depth: false,
        }),
        color_states: &[wgpu::ColorStateDescriptor {
            format: sc_desc.format,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }],
        primitive_topology: wgpu::PrimitiveTopology::TriangleList,
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[Vertex::desc()],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    render_pipeline
}
