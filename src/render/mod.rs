// let view = surface_texture
//     .texture
//     .create_view(&wgpu::TextureViewDescriptor {
//         format: Some(surface_format.add_srgb_suffix()),
//         ..Default::default()
//     });
//
// let mut encoder = device.create_command_encoder(&Default::default());
// world.term_renderer.render(&mut encoder, &view);
// queue.submit([encoder.finish()]);

use wgpu::util::DeviceExt;

pub fn byte_slice<T>(slice: &[T]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(slice.as_ptr().cast(), std::mem::size_of_val(slice)) }
}

pub fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: Option<wgpu::TextureFormat>,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    shader: wgpu::ShaderModuleDescriptor,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(shader);

    let depth_stencil = depth_format.map(|format| wgpu::DepthStencilState {
        format,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    });

    match color_format {
        Some(color_format) => device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: vertex_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState {
                        alpha: wgpu::BlendComponent::REPLACE,
                        color: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        }),
        None => device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: vertex_layouts,
                compilation_options: Default::default(),
            },
            fragment: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        }),
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct GlyphVertex {
    position: [f32; 3],
    uv: [f32; 2],
}

impl GlyphVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<GlyphVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Instance {
    position: [f32; 2],
    atlas_index: [f32; 2],
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![2 => Float32x2, 3 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

const VERTICES: &[GlyphVertex] = &[
    GlyphVertex {
        position: [0.5, 0.5, 0.0],
        uv: [1.0, 0.0],
    },
    GlyphVertex {
        position: [-0.5, 0.5, 0.0],
        uv: [0.0, 0.0],
    },
    GlyphVertex {
        position: [0.5, -0.5, 0.0],
        uv: [1.0, 1.0],
    },
    GlyphVertex {
        position: [0.5, -0.5, 0.0],
        uv: [1.0, 1.0],
    },
    GlyphVertex {
        position: [-0.5, 0.5, 0.0],
        uv: [0.0, 0.0],
    },
    GlyphVertex {
        position: [-0.5, -0.5, 0.0],
        uv: [0.0, 1.0],
    },
];

const INSTANCES: &[Instance] = &[
    Instance {
        position: [0.0, 0.0],
        atlas_index: [4.0, 4.0],
    },
    Instance {
        position: [1.0, 1.0],
        atlas_index: [6.0, 6.0],
    },
];

pub struct TermRenderer {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    instances: wgpu::Buffer,
    atlas_bind_group: wgpu::BindGroup,
}

impl TermRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        tileset_ppm: &[u8],
    ) -> Self {
        let (atlas, sampler) = parse_ppm(device, queue, tileset_ppm);

        let atlas_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("atlas bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let atlas_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("atlas bind group"),
            layout: &atlas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&atlas),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("term vertex buffer"),
            contents: byte_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let instances = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("term instance buffer"),
            contents: byte_slice(INSTANCES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("term pipeline layout"),
            bind_group_layouts: &[&atlas_bind_group_layout],
            push_constant_ranges: &[],
        });
        let shader = wgpu::include_wgsl!("shaders/term.wgsl");
        let pipeline = create_render_pipeline(
            device,
            &layout,
            Some(surface_format),
            None,
            &[GlyphVertex::desc(), Instance::desc()],
            shader,
        );

        Self {
            pipeline,
            vertices,
            instances,
            atlas_bind_group,
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("term render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.atlas_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertices.slice(..));
        render_pass.set_vertex_buffer(1, self.instances.slice(..));
        render_pass.draw(0..6, 0..INSTANCES.len() as u32);
    }
}

fn parse_ppm(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    mut tileset_ppm: &[u8],
) -> (wgpu::TextureView, wgpu::Sampler) {
    fn parse_usize(bytes: &[u8]) -> usize {
        let mut accum = 0;
        for byte in bytes.iter() {
            accum *= 10;
            accum += (*byte - b'0') as usize;
        }
        accum
    }

    debug_assert_eq!(
        &tileset_ppm[..3],
        b"P6\n",
        "some ppms have a P3 header but that format isn't supported here"
    );
    tileset_ppm = &tileset_ppm[3..];

    let mut ppm = tileset_ppm.split(u8::is_ascii_whitespace);
    let width = parse_usize(ppm.next().unwrap());
    let height = parse_usize(ppm.next().unwrap());
    debug_assert_eq!(width, height);

    let maxval = parse_usize(ppm.next().unwrap());
    assert!(maxval < 256);

    let rgba = ppm
        .next()
        .unwrap()
        .chunks(3)
        .flat_map(|chunk| [chunk[0], chunk[1], chunk[2], 255])
        .collect::<Vec<u8>>();
    debug_assert!(ppm.next().is_none());
    debug_assert!(rgba.len().is_multiple_of(4));

    let texture_size = wgpu::Extent3d {
        width: width as u32,
        height: height as u32,
        depth_or_array_layers: 1,
    };
    let texture_atlas = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label: Some("term texture atlas"),
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture_atlas,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width as u32),
            rows_per_image: Some(height as u32),
        },
        texture_size,
    );

    let view = texture_atlas.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    (view, sampler)
}
