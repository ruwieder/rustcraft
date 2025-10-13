use wgpu::*;
use wgpu::util::DeviceExt;
use cgmath::{Vector2, Vector3};
use crate::core::{mesh::Mesh, render::{camera::{Camera, UniformBuffer}, texture}};
use crate::core::World;
use crate::core::Vertex;

const SKYBOX: Color = Color{ r: 65.0 / 255.0, g: 200.0 / 255.0, b: 1.0, a: 1.0 };
const USE_GREEDY: bool = true;
const RENDER_LOGGING: bool = cfg!(debug_assertions);

pub struct Renderer {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
    pub render_pipeline: RenderPipeline,
    // pub vertex_buffer: Buffer,
    // pub index_buffer: Buffer,
    // pub index_count: u32,
    pub uniform_buffer: Buffer,
    pub uniform_bind_group: BindGroup,
    pub camera: Camera,
    depth_texture: Texture,
    // pub world: Box<World>,
    pub texture_bind_group: wgpu::BindGroup,
    pub texture: texture::Texture,
}

impl Renderer {
    pub async fn new(window: &'static winit::window::Window) -> Self {
        log::debug!("started renderer initializatioin...");
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    memory_hints: MemoryHints::default(),
                    experimental_features: ExperimentalFeatures::default(),
                    trace: Trace::default(),
                }
            )
            .await
            .unwrap();
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        
        let depth_texture = device.create_texture(&TextureDescriptor {
            label: Some("Depth Texture"),
            size: Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth24Plus,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        
        let uniform_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[UniformBuffer::default()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        
        let uniform_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("uniform_bind_group_layout"),
        });
        
        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("uniform_bind_group"),
        });
        
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        
        let diffuse_bytes = include_bytes!("../../../assets/textures/stone.png");
        let texture = texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "atlas.png").unwrap();
            
        let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });
        
        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });
        
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth24Plus,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        
        // let world = World::new();
        // let (vertices, indices) = if USE_GREEDY {
        //     world.build_mesh_greedy()
        // } else {
        //     world.build_mesh_naive()
        // };
        // println!("{} vertices generated", vertices.len());
        // let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
        //     label: Some("Vertex Buffer"),
        //     contents: bytemuck::cast_slice(&vertices),
        //     usage: BufferUsages::VERTEX,
        // });
        
        // let index_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
        //     label: Some("Index Buffer"),
        //     contents: bytemuck::cast_slice(&indices),
        //     usage: BufferUsages::INDEX,
        // });
        
        // let index_count = indices.len() as u32;
        
        let camera = Camera::new(
            Vector3::new(0.0, 0.0, 4.0),
            Vector2::new(0.0, 0.0),
            config.width as f32 / config.height as f32,
        );
        
        let uniform = camera.get_uniform();
        queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
        log::debug!("renderer initialized");
        Self {
            device,
            queue,
            surface,
            config,
            render_pipeline,
            // vertex_buffer,
            // index_buffer,
            // index_count,
            uniform_buffer,
            uniform_bind_group,
            camera,
            depth_texture,
            // world: Box::new(world),
            texture_bind_group,
            texture
        }
    }
    
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.camera.aspect = self.config.width as f32 / self.config.height as f32;
        
        // Recreate depth texture on resize
        self.depth_texture = self.device.create_texture(&TextureDescriptor {
            label: Some("Depth Texture"),
            size: Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth24Plus,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
    }
    
    pub fn new_render(&self, world: &World) -> Result<(), SurfaceError> {
        if RENDER_LOGGING {log::trace!("started render...");}
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
    
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(SKYBOX), // Use your skybox color
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });
    
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
            
            let mut count_rendered = 0;
            for (key, mesh) in &world.meshes {
                
                let chunk = world.chunks.get(key);
                if chunk.is_none() || chunk.unwrap().is_dirty {
                    if RENDER_LOGGING {
                        log::trace!("chunk at {key:?} skipped: (is_none: {})", chunk.is_none());
                    }
                    continue;
                }
                if mesh.vertex_buffer.is_none() || mesh.index_buffer.is_none() {
                    if RENDER_LOGGING {
                        log::trace!(
                            "mesh at {key:?} skipped: (vertex_buffer.is_none: {}, index_buffer.is_none: {})",
                            mesh.vertex_buffer.is_none(), mesh.index_buffer.is_none()
                        );
                    }
                    continue;
                }
                if let Some(b) = mesh.index_buffer.as_ref() && b.size() != 0 {
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.as_ref().unwrap().slice(..));
                    render_pass.set_index_buffer(mesh.index_buffer.as_ref().unwrap().slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                    count_rendered += 1;
                }
            }
            if RENDER_LOGGING {log::trace!("rendered {count_rendered} meshes");}
        }
    
        self.queue.submit(std::iter::once(encoder.finish()));        
        output.present();
        if RENDER_LOGGING {log::trace!("render pass done");}
        Ok(())
    }    
    // pub fn render(&mut self) -> Result<(), SurfaceError> {
    //     let frame = self.surface.get_current_texture()?;
    //     let view = frame.texture.create_view(&TextureViewDescriptor::default());
    //     let depth_view = self.depth_texture.create_view(&TextureViewDescriptor::default());

    //     let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
    //         label: Some("Render Encoder"),
    //     });
        
    //     {
    //         let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
    //             label: Some("Render Pass"),
    //             color_attachments: &[Some(RenderPassColorAttachment {
    //                 view: &view,
    //                 resolve_target: None,
    //                 ops: Operations {
    //                     load: LoadOp::Clear(SKYBOX),
    //                     store: StoreOp::Store,
    //                 },
    //                 depth_slice: None,
    //             })],
    //             depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
    //                 view: &depth_view,
    //                 depth_ops: Some(Operations {
    //                     load: LoadOp::Clear(1.0),
    //                     store: StoreOp::Store,
    //                 }),
    //                 stencil_ops: None,
    //             }),
    //             timestamp_writes: None,
    //             occlusion_query_set: None,
    //         });
            
    //         render_pass.set_pipeline(&self.render_pipeline);
    //         render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
    //         render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
    //         render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    //         render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
    //         render_pass.draw_indexed(0..self.index_count, 0, 0..1);
    //     }
        
    //     self.queue.submit(std::iter::once(encoder.finish()));
    //     frame.present();
        
    //     Ok(())
    // }
    
    pub fn update_camera(&mut self, dt: f64, movement: (f32, f32, f32), mouse_delta: (f32, f32)) {
        if  movement.0 == 0.0 && movement.1 == 0.0 && movement.2 == 0.0 
          && mouse_delta.0 == 0.0 && mouse_delta.1 == 0.0 {
            return;
        }
        self.camera.update(dt, movement, mouse_delta);
        let uniform = self.camera.get_uniform();
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
    
    pub fn update_mesh_buffers(&self, mesh: &mut Mesh) {
        if !mesh.is_dirty {
            return;
        };
        
        mesh.vertex_buffer = Some(self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Vertex Buffer")),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        ));
        mesh.index_buffer = Some(self.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Index Buffer")),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        ));
        mesh.index_count = mesh.indices.len() as u32;
        mesh.is_dirty = false;
        #[cfg(debug_assertions)]
        log::trace!("loaded chunk mesh buffers of {} indices", mesh.index_count);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // Wait for the GPU to finish all work before dropping resources
        log::info!("dropping renderer...");
        let _ = self.device.poll(wgpu::PollType::Poll);
        log::info!("done");
    }
}