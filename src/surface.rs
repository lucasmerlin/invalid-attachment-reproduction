use std::borrow::Cow;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use wgpu::SamplerDescriptor;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![0 => Float32x2];

    const fn vertex_buffer_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &self::Vertex::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Dot {
    position: [f32; 2],
    radius: f32,
    hardness: f32,
    color: [f32; 4],
}

impl Dot {
    const ATTRIBUTES: &'static [wgpu::VertexAttribute] = &wgpu::vertex_attr_array![1 => Float32x2, 2 => Float32, 3 => Float32, 4 => Float32x4];

    const fn vertex_buffer_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Dot>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &self::Dot::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    frame: u32,
    _padding1: u32,
    _padding2: u32,
    _padding3: u32,
}


static VERTICES: [Vertex; 6] = [
    Vertex { position: [0.0, 0.0] },
    Vertex { position: [1.0, 0.0] },
    Vertex { position: [1.0, 1.0] },
    Vertex { position: [1.0, 1.0] },
    Vertex { position: [0.0, 1.0] },
    Vertex { position: [0.0, 0.0] },
];

static TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

pub struct GlobalSurface {
    pub device: Arc<wgpu::Device>,

    pub queue: Arc<wgpu::Queue>,

    pub vertex_buffer: wgpu::Buffer,

    pub render_pipeline: wgpu::RenderPipeline,

    pub texture_desc: wgpu::TextureDescriptor<'static>,
}


impl GlobalSurface {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });


        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("dot_shader.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Surface Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });


        let texture_size = 1024u32;

        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: texture_size,
                height: texture_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
            ,
            label: None,
            view_formats: &[],
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::vertex_buffer_desc(), Dot::vertex_buffer_desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: texture_desc.format,

                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent::OVER,
                        }),

                        write_mask: wgpu::ColorWrites::ALL,
                    })
                ],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            device,

            queue,

            vertex_buffer,

            render_pipeline,

            texture_desc,
        }
    }
}


pub struct HpSurface {
    pub global: Arc<GlobalSurface>,

    pub instances: Vec<Dot>,

    pub instance_buffer: wgpu::Buffer,

    pub texture: wgpu::Texture,

    pub texture_view: wgpu::TextureView,

    pub sampler: wgpu::Sampler,
}

impl HpSurface {
    pub fn new(global: Arc<GlobalSurface>) -> Self {
        let instances = vec![
            Dot {
                position: [0.5, 0.5],
                radius: 0.1,
                hardness: 0.5,
                color: [1.0, 0.0, 0.0, 1.0],
            },
        ];

        let instance_buffer = global.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let texture = global.device.create_texture(&global.texture_desc);

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = global.device.create_sampler(&SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            global,
            instances,
            instance_buffer,
            texture,
            texture_view,
            sampler,
        }
    }

    pub fn render(&self) {
        let mut encoder = self.global.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(
                    wgpu::RenderPassColorAttachment {
                        view: &self.texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                            store: true,
                        },
                    }
                )],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.global.render_pipeline);
            render_pass.set_vertex_buffer(0, self.global.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }

        self.global.queue.submit(Some(encoder.finish()));
    }
}
