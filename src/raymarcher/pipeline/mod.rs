use std::marker::PhantomData;

use nalgebra as na;
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
    pub x: [f32; 4],
    pub y: [f32; 4],
    pub z: [f32; 4],
    pub w: [f32; 4],
}
impl From<[[f32; 4]; 4]> for CameraUniform {
    fn from(value: [[f32; 4]; 4]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2],
            w: value[3],
        }
    }
}
impl From<na::Matrix4<f32>> for CameraUniform {
    fn from(value: na::Matrix4<f32>) -> Self {
        Self::from(Into::<[[f32; 4]; 4]>::into(value))
    }
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
    camera_bind_group_layout: &wgpu::BindGroupLayout,
    camera_uniform: CameraUniform,
) -> BindGroup<CameraUniform> {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[camera_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: camera_bind_group_layout,
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SphereUniform {
    pub pos: [f32; 3],
    pub rad: f32,
}
pub fn sphere_bindgroup_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
        label: Some("Sphere Bind Group Layout"),
    })
}
pub fn sphere_bindgroup(
    device: &wgpu::Device,
    sphere_bind_group_layout: &wgpu::BindGroupLayout,
    sphere_uniform: SphereUniform,
) -> BindGroup<SphereUniform> {
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Sphere Buffer"),
        contents: bytemuck::cast_slice(&[sphere_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: sphere_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
        label: Some("SPhere Bind Group"),
    });
    BindGroup {
        bindgroup,
        buffer,
        phantom: PhantomData,
    }
}

pub struct BindGroupLayouts {
    pub camera: wgpu::BindGroupLayout,
    pub sphere: wgpu::BindGroupLayout,
}
impl BindGroupLayouts {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            camera: camera_bindgroup_layout(device),
            sphere: sphere_bindgroup_layout(device),
        }
    }
}

pub fn render_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    bind_group_layout: &BindGroupLayouts,
) -> wgpu::RenderPipeline {
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout.camera, &bind_group_layout.sphere],
        push_constant_ranges: &[],
    });

    let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
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
