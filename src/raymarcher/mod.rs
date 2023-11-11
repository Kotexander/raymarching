mod pipeline;
pub mod wgpu_context;

use nalgebra as na;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use wgpu_context::WgpuContext;

pub struct Camera {
    pos: na::Point3<f32>,
    dir: na::Vector3<f32>,
}
impl Camera {
    fn build(&self) -> na::Matrix4<f32> {
        na::Matrix4::face_towards(&self.pos, &(self.pos + self.dir), &na::Vector3::y())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Controller {
    pub right: bool,
    pub left: bool,
    pub down: bool,
    pub up: bool,
    pub forwards: bool,
    pub backwards: bool,
}

pub struct RayMarcher<W>
where
    W: HasRawWindowHandle + HasRawDisplayHandle,
{
    pub wgpu_ctx: WgpuContext<W>,
    pub camera: Camera,
    camera_bindgroup: pipeline::BindGroup<pipeline::CameraUniform>,
    sphere_bindgroup: pipeline::BindGroup<pipeline::SphereUniform>,
    pipeline: wgpu::RenderPipeline,
    mesh: pipeline::Mesh<pipeline::Vertex>,

    pub controller: Controller,
}
impl<W: HasRawWindowHandle + HasRawDisplayHandle> RayMarcher<W> {
    pub async fn new(window: W, size: (u32, u32)) -> Self {
        let wgpu_ctx = WgpuContext::new(window, size).await;
        let camera = Camera {
            pos: na::point![0.0, 0.0, 0.0],
            dir: na::vector![0.0, 0.0, 1.0],
        };

        // let camera_bind_group_layout = pipeline::camera_bind_group_layout(&wgpu_ctx.device);
        let bind_group_layouts = pipeline::BindGroupLayouts::new(&wgpu_ctx.device);

        let camera_bindgroup = pipeline::camera_bindgroup(
            &wgpu_ctx.device,
            &bind_group_layouts.camera,
            na::Matrix4::identity().into(),
        );
        let sphere_bindgroup = pipeline::sphere_bindgroup(
            &wgpu_ctx.device,
            &bind_group_layouts.sphere,
            pipeline::SphereUniform {
                pos: [0.0, 0.0, 3.0],
                rad: 0.5,
            },
        );
        let pipeline = pipeline::render_pipeline(
            &wgpu_ctx.device,
            wgpu_ctx.config.format,
            &bind_group_layouts,
        );
        let mesh = pipeline::new_fullscreen_quad(&wgpu_ctx.device);

        let controller = Controller::default();

        Self {
            wgpu_ctx,
            camera,
            camera_bindgroup,
            sphere_bindgroup,
            pipeline,
            mesh,
            controller,
        }
    }
    pub fn update(&mut self, dt: f32) {
        let mut dir = na::Vector3::<f32>::zeros();
        if self.controller.up {
            dir += na::Vector3::y();
        }
        if self.controller.down {
            dir -= na::Vector3::y();
        }
        if self.controller.right {
            dir += na::Vector3::x();
        }
        if self.controller.left {
            dir -= na::Vector3::x();
        }
        if self.controller.forwards {
            dir += na::Vector3::z();
        }
        if self.controller.backwards {
            dir -= na::Vector3::z();
        }
        if dir.magnitude_squared() != 0.0 {
            dir = dir.normalize();
            self.camera.pos += dir * dt;
        }
    }
    fn aspect(&self) -> f32 {
        // self.wgpu_ctx.config.width as f32 / self.wgpu_ctx.config.height as f32
        self.wgpu_ctx.config.height as f32 / self.wgpu_ctx.config.width as f32
    }
    fn camera_mat(&self) -> na::Matrix4<f32> {
        self.camera
            .build()
            .append_nonuniform_scaling(&na::vector![1.0, self.aspect(), 1.0])
    }
    fn update_graphics(&self) {
        self.camera_bindgroup
            .update(&self.wgpu_ctx.queue, self.camera_mat().into());
    }
    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        self.update_graphics();

        // get window's view
        let output = self.wgpu_ctx.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Output View"),
            ..Default::default()
        });

        let mut encoder =
            self.wgpu_ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.1,
                                b: 0.1,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.camera_bindgroup.bindgroup, &[]);
            render_pass.set_bind_group(1, &self.sphere_bindgroup.bindgroup, &[]);
            self.mesh.draw(&mut render_pass);
        }
        // submit will accept anything that implements IntoIter
        self.wgpu_ctx
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
