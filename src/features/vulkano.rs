use std::sync::Arc;
use std::collections::HashMap;
use std::mem;

use render::DrawList;
use render::Update;
use render::Vertex;
use render::Command;

use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::pipeline::viewport::Scissor;
use vulkano::device::*;
use vulkano::pipeline::*;
use vulkano::descriptor::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::framebuffer::*;
use vulkano::image::*;
use vulkano::format::*;
use vulkano::sampler::*;
use vulkano::pipeline::viewport::Viewport;
use vulkano::command_buffer::DynamicState;

#[allow(dead_code, unsafe_code)]
mod shaders {
    pub mod vertex_shader {
        #[derive(VulkanoShader)]
        #[ty = "vertex"]
        #[path = "src/features/vulkano.vs"]
        struct Dummy;
    }

    pub mod fragment_shader {
        #[derive(VulkanoShader)]
        #[ty = "fragment"]
        #[path = "src/features/vulkano.fs"]
        struct Dummy;
    }
}

impl_vertex!(Vertex, pos, uv, color, mode);

enum Texture {
    Immutable(Arc<ImmutableImage<R8G8B8A8Unorm>>),
    Atlas(Arc<StorageImage<R8G8B8A8Unorm>>),
}

impl Texture {
    fn access(&self) -> Arc<ImageViewAccess+Send+Sync> {
        match self {
            &Texture::Immutable(ref im) => im.clone(),
            &Texture::Atlas(ref im) => im.clone(),
        }
    }
}

pub struct Renderer {
    pipeline: Box<Arc<GraphicsPipelineAbstract+Send+Sync>>,
    texture_uploads: CpuBufferPool<u8>,
    textures: HashMap<usize, Texture>,
    sampler: Arc<Sampler>,
    tex_descs: FixedSizeDescriptorSetsPool<Arc<GraphicsPipelineAbstract+Send+Sync>>,
}

impl Renderer {
    /// Construct a new empty `Renderer`.
    pub fn new<
        'a,
        L: RenderPassDesc + RenderPassAbstract + Send + Sync + 'static
    > (
        device: Arc<Device>,
        subpass: Subpass<L>
    ) -> Self {
        let sampler = Sampler::new(
            device.clone(), 
            Filter::Linear,
            Filter::Linear, 
            MipmapMode::Nearest,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            0.0, 1.0, 0.0, 0.0
        ).unwrap();

        let vertex_shader = 
            shaders::vertex_shader::Shader::load(device.clone())
            .expect("failed to create shader module");

        let fragment_shader = 
            shaders::fragment_shader::Shader::load(device.clone())
            .expect("failed to create shader module");

        let pipeline = Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vertex_shader.main_entry_point(), ())
            .depth_stencil_disabled()
            .triangle_list()
            .front_face_clockwise()
            //.cull_mode_back()
            .viewports_scissors_dynamic(1)
            .fragment_shader(fragment_shader.main_entry_point(), ())
            .blend_alpha_blending()
            .render_pass(subpass)
            .build(device.clone())
            .unwrap());

        let tex_descs = FixedSizeDescriptorSetsPool::new(pipeline.clone() as Arc<_>, 0);

        Renderer {
            pipeline: Box::new(pipeline),
            textures: HashMap::new(),
            texture_uploads: CpuBufferPool::upload(device.clone()),
            sampler,
            tex_descs,            
        }
    }

    pub fn upload(
        &mut self,
        device: Arc<Device>,
        graphics_queue: Arc<Queue>,
        drawlist: &mut DrawList,
        mut cmd: AutoCommandBufferBuilder
    ) -> AutoCommandBufferBuilder {
        for update in mem::replace(&mut drawlist.updates, Vec::new()) {
            match update {
                Update::TextureSubresource { id, offset, size, data } => {
                    let buffer = self.texture_uploads.chunk(data.into_iter()).unwrap();
                    let texture = match &self.textures[&id] {
                        &Texture::Immutable(_) => panic!("Updating immutable texture!"),
                        &Texture::Atlas(ref im) => im.clone(),
                    };

                    cmd = cmd.copy_buffer_to_image_dimensions(
                        buffer, 
                        texture, 
                        [offset[0], offset[1], 0], 
                        [size[0], size[1], 1], 
                        0, 
                        1, 
                        0
                    ).unwrap()
                },
                Update::Texture { id, size, data, atlas } => {
                    if atlas {
                        let texture = StorageImage::with_usage(
                            device.clone(), 
                            Dimensions::Dim2d{ width: size[0], height: size[1] }, 
                            R8G8B8A8Unorm, 
                            ImageUsage {
                                transfer_destination: true,
                                sampled: true,
                                .. ImageUsage::none()
                            }, 
                            vec![graphics_queue.family()]
                        ).unwrap();

                        self.textures.insert(id, Texture::Atlas(texture));
                    } else {
                        let (texture, done) = ImmutableImage::from_iter(
                            data.into_iter(),
                            Dimensions::Dim2d{ width: size[0], height: size[1] },
                            R8G8B8A8Unorm,
                            graphics_queue.clone()
                        ).unwrap();

                        // Wait until texture upload is done. 
                        // This is not very efficient but it gets the job done..
                        drop(done);

                        self.textures.insert(id, Texture::Immutable(texture));
                    }
                },
            }
        }

        cmd
    }

    pub fn draw(
        &mut self, 
        device: Arc<Device>,
        viewport: [f32; 4],
        drawlist: DrawList,
        mut cmd: AutoCommandBufferBuilder
    ) -> AutoCommandBufferBuilder {

        let DrawList {
            updates,
            vertices,
            commands
        } = drawlist;

        assert!(updates.len() == 0);

        let current_viewport = Viewport {
            origin: [viewport[0], viewport[1]],
            dimensions: [viewport[2]-viewport[0], viewport[3]-viewport[1]],
            depth_range: 0.0 .. 1.0,
        };

        let mut current_scissor = Scissor {
            origin: [viewport[0] as i32, viewport[1] as i32],
            dimensions: [(viewport[2]-viewport[0]) as u32, (viewport[3]-viewport[1]) as u32],
        };

        let desc_cache = Arc::new(self.tex_descs.next()
            .add_sampled_image(self.textures[&0].access(), self.sampler.clone()).unwrap()
            .build().unwrap());

        let tex_descs = &mut self.tex_descs;

        for command in commands {
            match command {
                Command::Nop => {
                },
                
                Command::Clip{ scissor } => {
                    current_scissor = Scissor {
                        origin: [
                            scissor.left as i32, 
                            scissor.top as i32,
                        ],
                        dimensions: [
                            (scissor.right-scissor.left) as u32, 
                            (scissor.bottom-scissor.top) as u32,
                        ],
                    };
                },
                
                Command::Colored{ offset, count } => {
                    if count > 0 {
                        let vbuf = CpuAccessibleBuffer::from_iter(
                            device.clone(), 
                            BufferUsage::vertex_buffer(), 
                            (&vertices[offset..offset+count]).into_iter().cloned()
                        ).unwrap();

                        cmd = cmd
                            .draw(self.pipeline.clone(),
                                DynamicState {
                                    line_width: None,
                                    viewports: Some(vec![current_viewport.clone()]),
                                    scissors: Some(vec![current_scissor.clone()]),
                                },
                                vec![vbuf],
                                desc_cache.clone(), 
                                ())
                            .unwrap();
                    }
                },

                Command::Textured{ texture, offset, count } => {
                    if count > 0 {
                        let vbuf = CpuAccessibleBuffer::from_iter(
                            device.clone(), 
                            BufferUsage::vertex_buffer(), 
                            (&vertices[offset..offset+count]).into_iter().cloned()
                        ).unwrap();

                        let texture = self.textures[&texture].access();
                        let sampler = self.sampler.clone();
                        let desc_image = Arc::new(tex_descs.next()
                            .add_sampled_image(texture, sampler).unwrap()
                            .build().unwrap());

                        cmd = cmd
                            .draw(self.pipeline.clone(),
                                DynamicState {
                                    line_width: None,
                                    viewports: Some(vec![current_viewport.clone()]),
                                    scissors: Some(vec![current_scissor.clone()]),
                                },
                                vec![vbuf],
                                desc_image, 
                                ())
                            .unwrap();
                    }
                },
            }
        }

        cmd
    }
}
