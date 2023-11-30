use std::marker::PhantomData;

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
}
impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct BindGroup<D> {
    pub buffer: wgpu::Buffer,
    pub bindgroup: wgpu::BindGroup,
    phantom: PhantomData<D>,
}
impl<D: bytemuck::Zeroable + bytemuck::Pod> BindGroup<D> {
    pub fn update(&self, queue: &wgpu::Queue, data: D) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
    }
}

pub struct Mesh<V: bytemuck::Pod + bytemuck::Zeroable> {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    phantom: PhantomData<V>,
}
impl<V: bytemuck::Pod + bytemuck::Zeroable> Mesh<V> {
    pub fn new(device: &wgpu::Device, vertices: &[V], indices: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            phantom: PhantomData,
        }
    }
    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}

pub fn new_fullscreen_quad(device: &wgpu::Device) -> Mesh<Vertex> {
    const VERTICIES: [Vertex; 4] = [
        Vertex {
            position: [1.0, 1.0],
        },
        Vertex {
            position: [1.0, -1.0],
        },
        Vertex {
            position: [-1.0, -1.0],
        },
        Vertex {
            position: [-1.0, 1.0],
        },
    ];
    const INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];
    Mesh::new(device, &VERTICIES, &INDICES)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub matrix: [[f32; 4]; 4],
}

pub fn camera_bindgroup_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("Camera Bind Group Layout"),
    })
}
pub fn camera_bindgroup(
    device: &wgpu::Device,
    camera_bindgroup_layout: &wgpu::BindGroupLayout,
    camera_uniform: CameraUniform,
) -> BindGroup<CameraUniform> {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[camera_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: camera_bindgroup_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        label: Some("Camera Bind Group"),
    });
    BindGroup {
        bindgroup,
        buffer,
        phantom: PhantomData,
    }
}

pub fn texture_bindgroup_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
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
        label: Some("Texture Bind Group Layout"),
    })
}
pub fn texture_bindgroup(
    device: &wgpu::Device,
    texture_bindgroup_layout: &wgpu::BindGroupLayout,
    view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: texture_bindgroup_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
        label: Some("Raymarcher Bind Group"),
    })
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SettingsUniform {
    pub max_steps: i32,
    pub epsilon: f32,
    pub max_dist: f32,

    pub sun_size: f32,
    pub sun_dir: [f32; 3],
    pub sun_sharpness: f32,

    pub alpha: f32,

    pub time: f32,
    pub scene: u32,

    pub _padding: u32,
}
impl SettingsUniform {
    pub fn set_mandelbulb(mut self) -> Self {
        self.scene = 0;
        self.max_steps = 100;
        self.epsilon = 0.002;
        self
    }
    pub fn set_mengersponge(mut self) -> Self {
        self.scene = 1;
        self.max_steps = 500;
        self.epsilon = 0.0001;
        self
    }
    pub fn set_mandelbulb_mut(&mut self) {
        *self = self.set_mandelbulb();
    }
    pub fn set_mengersponge_mut(&mut self) {
        *self = self.set_mengersponge();
    }
}
impl Default for SettingsUniform {
    fn default() -> Self {
        Self {
            max_steps: 00,
            epsilon: 0.000,
            max_dist: 10.0,
            sun_size: 0.005,
            sun_dir: [0.0, 1.0, 0.0],
            sun_sharpness: 2.0,
            alpha: 0.1,
            time: 0.0,
            scene: 0,
            _padding: 0,
        }
        .set_mandelbulb()
    }
}

pub fn settings_bindgroup_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("Settings Bind Group Layout"),
    })
}
pub fn settings_bindgroup(
    device: &wgpu::Device,
    settings_bindgroup_layout: &wgpu::BindGroupLayout,
    settings_uniform: SettingsUniform,
) -> BindGroup<SettingsUniform> {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Settings Buffer"),
        contents: bytemuck::cast_slice(&[settings_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: settings_bindgroup_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        label: Some("Settings Bind Group"),
    });
    BindGroup {
        bindgroup,
        buffer,
        phantom: PhantomData,
    }
}

pub fn raymarcher_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    camera_bindgroup_layout: &wgpu::BindGroupLayout,
    settings_bindgroup_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Raymarcher Pipeline Layout"),
        bind_group_layouts: &[camera_bindgroup_layout, settings_bindgroup_layout],
        push_constant_ranges: &[],
    });

    let shader = device.create_shader_module(wgpu::include_wgsl!("raymarcher.wgsl"));

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Raymarcher Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

pub fn fullscreen_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    texture_bindgroup_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Fullscreen Pipeline Layout"),
        bind_group_layouts: &[texture_bindgroup_layout],
        push_constant_ranges: &[],
    });

    let shader = device.create_shader_module(wgpu::include_wgsl!("fullscreen.wgsl"));

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Fullscreen Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}
