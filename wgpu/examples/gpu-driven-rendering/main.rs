#[path = "../framework.rs"]
mod framework;
mod shapes;

use std::{borrow::Cow, f32::consts, future::Future, mem, pin::Pin, task, vec::Vec};
use wgpu::util::DeviceExt;

// Convert color from 0-255 format to 0-1  ([255, 127, 0, 255] -> [1.0, 0,4980392156862745, 0, 1.0])
fn create_color(clr: [u8; 4]) -> [f32; 4] {
    let mut result_clr: [f32; 4] = [0.0; 4];
    for i in 0..clr.len() {
        result_clr[i] = (clr[i] as f32) / 255.0;
    }
    result_clr
}

/// A wrapper for `pop_error_scope` futures that panics if an error occurs.
///
/// Given a future `inner` of an `Option<E>` for some error type `E`,
/// wait for the future to be ready, and panic if its value is `Some`.
///
/// This can be done simpler with `FutureExt`, but we don't want to add
/// a dependency just for this small case.
struct ErrorFuture<F> {
    inner: F,
}
impl<F: Future<Output = Option<wgpu::Error>>> Future for ErrorFuture<F> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<()> {
        let inner = unsafe { self.map_unchecked_mut(|me| &mut me.inner) };
        inner.poll(cx).map(|error| {
            if let Some(e) = error {
                panic!("Rendering {}", e);
            }
        })
    }
}

struct Example {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    indirect_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    pipeline_wire: Option<wgpu::RenderPipeline>,
}

impl Example {
    fn generate_matrix(aspect_ratio: f32) -> glam::Mat4 {
        let projection = glam::Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0);
        let view = glam::Mat4::look_at_rh(
            glam::Vec3::new(0.0, -8.0, 3.0),
            glam::Vec3::ZERO,
            glam::Vec3::Z,
        );
        projection * view
    }
}

impl framework::Example for Example {
    fn optional_features() -> wgt::Features {
        wgt::Features::MULTI_DRAW_INDIRECT | wgt::Features::POLYGON_MODE_LINE
    }

    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        // Create the vertex and index buffers
        let vertex_size = mem::size_of::<shapes::Vertex>();
        let (mut vertex_data1, index_data1) = shapes::cube::create_vertices();
        let (mut vertex_data2, mut index_data2) = shapes::sphere::generate_vertices();

        let translation_matrices = [
            [2.0, 0.0, 0.0, 0.0], // cube
            [-2.0, 0.0, 0.0, 0.0], // sphere
        ];

        shapes::translate_vertex_data(&mut vertex_data1, &translation_matrices[0]);
        shapes::translate_vertex_data(&mut vertex_data2, &translation_matrices[1]);

        let vertex_data: Vec<shapes::Vertex> = [&vertex_data1[..], &vertex_data2[..]].concat();
        //let index_data: Vec<u8> = [bytemuck::cast_slice(&index_data1), bytemuck::cast_slice(&index_data2)].concat();
        let index_data: Vec<u16> = shapes::merge_index_data(&index_data1, &mut index_data2, vertex_data1.len() as u16);

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(4*4),
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create other resources
        let mx_total = Self::generate_matrix(config.width as f32 / config.height as f32);
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(mx_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create cube color
        let color: &[f32; 4] = &create_color([210, 191, 85, 255]); // create color that has values 0-1 instead of 0-255
        let color_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(color),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create cube indirect buffer
        let cube_indirect_data = &wgpu::util::DrawIndexedIndirect {
            vertex_count: index_data1.len() as u32,
            instance_count: 1,
            base_index: 0,
            vertex_offset: 0,
            base_instance: 0,
        };

        // Create sphere indirect buffer
        let sphere_indirect_data = &wgpu::util::DrawIndexedIndirect {
            vertex_count: index_data2.len() as u32,
            instance_count: 1,
            base_index: 0,
            vertex_offset: 0,
            base_instance: 0,
        };

        let indirect_data = &[cube_indirect_data.as_bytes(), sphere_indirect_data.as_bytes()].concat();
        //let indirect_data = cube_indirect_data.as_bytes();
        println!("{}", indirect_data.len());

        // Create one indirect buffer
        let indirect_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Indirect Buffer"),
            contents: indirect_data,
            usage: wgpu::BufferUsages::INDIRECT,
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: color_buf.as_entire_binding(),
                },
            ],
            label: None,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        }];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(config.format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                //topology: wgpu::PrimitiveTopology::PointList,
                //polygon_mode: wgpu::PolygonMode::Point,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let pipeline_wire = if device.features().contains(wgt::Features::POLYGON_MODE_LINE) {
            let pipeline_wire = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_wire",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                operation: wgpu::BlendOperation::Add,
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            },
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Line,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });
            Some(pipeline_wire)
        } else {
            None
        };

        // Done
        Example {
            vertex_buf,
            index_buf,
            index_count: index_data1.len() + index_data2.len(),
            bind_group,
            uniform_buf,
            indirect_buf,
            pipeline,
            pipeline_wire,
        }
    }

    fn update(&mut self, _event: winit::event::WindowEvent) {
        //empty
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let mx_total = Self::generate_matrix(config.width as f32 / config.height as f32);
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(mx_ref));
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        spawner: &framework::Spawner,
    ) {
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
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
                })],
                depth_stencil_attachment: None,
            });
            rpass.push_debug_group("Prepare data for draw.");
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            //rpass.draw_indexed(
            //    0..self.index_count as u32, //indices
            //    0, // base_vertex
            //    0..1 // instances
            //);
            rpass.multi_draw_indexed_indirect(
                &self.indirect_buf, // indirect_buffer
                0, // indirect_offset
                2, // count
            );
            if let Some(ref pipe) = self.pipeline_wire {
                rpass.set_pipeline(pipe);
                rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
            }
        }

        queue.submit(Some(encoder.finish()));

        // If an error occurs, report it and panic.
        spawner.spawn_local(ErrorFuture {
            inner: device.pop_error_scope(),
        });
    }
}

fn main() {
    framework::run::<Example>("gpu driven rendering");
}
