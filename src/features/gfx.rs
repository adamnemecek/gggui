use std::collections::HashMap;
use std::mem;

use gfx;
use gfx::traits::FactoryExt;

use render::DrawList;
use render::Update;
use render::Command;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_TexCoord",
        color: [f32; 4] = "a_Color",
        mode: u32 = "a_Mode",
    }

    pipeline pipeline_gggui {
        vbuf: gfx::VertexBuffer<Vertex> 
            = (),

        color: gfx::TextureSampler<[f32; 4]> 
            = "t_Color",

        //scissor: gfx::Scissor 
        //    = (),

        out_color: gfx::BlendTarget<gfx::format::Rgba8> 
            = ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    }
}

pub struct Renderer<R: gfx::Resources> {
    pipeline: gfx::PipelineState<R, pipeline_gggui::Meta>,
    textures: HashMap<usize, (
        gfx::handle::Texture<R, gfx::format::R8_G8_B8_A8>,
        gfx::handle::ShaderResourceView<R, [f32;4]>)>,
    sampler: gfx::handle::Sampler<R>,
}

impl<R: gfx::Resources> Renderer<R> {
    /// Construct a new empty `Renderer`.
    pub fn new<F: gfx::Factory<R>>(factory: &mut F) -> Self {
        Renderer {
            pipeline: factory.create_pipeline_simple(
                include_bytes!("gfx.vs"),
                include_bytes!("gfx.fs"),
                pipeline_gggui::new()
            ).unwrap(),

            textures: HashMap::new(),

            sampler: factory.create_sampler(gfx::texture::SamplerInfo::new(
                gfx::texture::FilterMethod::Scale,
                gfx::texture::WrapMode::Clamp
            )),
        }
    }

    fn update<C: gfx::CommandBuffer<R>, F: gfx::Factory<R>>(
        &mut self, 
        fac: &mut F, 
        enc: &mut gfx::Encoder<R, C>, 
        drawlist: &mut DrawList) {
        for update in mem::replace(&mut drawlist.updates, vec![]) {
            match update {
                Update::TextureSubresource{ id, offset, size, data } => {
                    println!("subres");
                    let converted_data: Vec<[u8; 4]> = 
                        data.chunks(4).map(|c: &[u8]| [c[0], c[1], c[2], c[3]]).collect();
                    enc.update_texture::<gfx::format::R8_G8_B8_A8, gfx::format::Rgba8>(
                        &self.textures[&id].0, 
                        None, 
                        gfx::texture::NewImageInfo {
                            xoffset: offset[0] as _,
                            yoffset: offset[1] as _,
                            zoffset: 0,
                            width: size[0] as _,
                            height: size[1] as _,
                            depth: 0,
                            format: (),
                            mipmap: 0,
                        }, 
                        &converted_data
                    ).expect("texture update failed");
                },

                Update::Texture{ id, size, data, .. } => {
                    println!("tex");
                    let texture = fac.create_texture::<gfx::format::R8_G8_B8_A8>(
                        gfx::texture::Kind::D2(size[0] as _, size[1] as _, gfx::texture::AaMode::Single), 
                        1, 
                        gfx::memory::Bind::SHADER_RESOURCE | gfx::memory::Bind::TRANSFER_DST, 
                        gfx::memory::Usage::Dynamic, 
                        Some(gfx::format::ChannelType::Unorm)
                    ).unwrap();

                    let view = fac.view_texture_as_shader_resource::<gfx::format::Rgba8>(
                        &texture, 
                        (0, 1), 
                        gfx::format::Swizzle::new()
                    ).unwrap();

                    let converted_data: Vec<[u8; 4]> = 
                        data.chunks(4).map(|c: &[u8]| [c[0], c[1], c[2], c[3]]).collect();
                    enc.update_texture::<gfx::format::R8_G8_B8_A8, gfx::format::Rgba8>(
                        &texture, 
                        None, 
                        gfx::texture::NewImageInfo {
                            xoffset: 0,
                            yoffset: 0,
                            zoffset: 0,
                            width: size[0] as _,
                            height: size[1] as _,
                            depth: 1,
                            format: (),
                            mipmap: 0,
                        }, 
                        &converted_data
                    ).expect("texture update failed");

                    self.textures.insert(id, (texture, view));
                },
            }
        }
    }

    pub fn draw<C: gfx::CommandBuffer<R>, F: gfx::Factory<R>>(
        &mut self, 
        fac: &mut F,
        enc: &mut gfx::Encoder<R, C>, 
        out: &gfx::handle::RenderTargetView<R, gfx::format::Rgba8>, 
        mut drawlist: DrawList) { 

        self.update(fac, enc, &mut drawlist);

        let DrawList {
            updates,
            vertices,
            commands
        } = drawlist;

        assert!(updates.len() == 0);

        let (width,height,_depth,_samples) = out.get_dimensions();

        // convert to gfx vertices. a bit unfortunate, but hopefully this gets optimized out.
        let vertices = vertices.into_iter().map(|v| Vertex {
            pos: v.pos,
            uv: v.uv,
            color: v.color,
            mode: v.mode,
        }).collect::<Vec<_>>();

        let mut current_scissor = gfx::Rect {
            x: 0, 
            y: 0,
            w: width, 
            h: height,
        };

        for command in commands {
            match command {
                Command::Nop => {
                },
                
                Command::Clip{ scissor } => {
                    current_scissor = gfx::Rect {
                        x: scissor.left as _, 
                        y: scissor.top as _,
                        w: (scissor.right-scissor.left) as _, 
                        h: (scissor.bottom-scissor.top) as _,
                    };
                },
                
                Command::Colored{ offset, count } => {
                    if count > 0 {
                        let (vbuf, slice) = fac.create_vertex_buffer_with_slice(&vertices[offset..offset+count], ());

                        enc.draw(&slice, &self.pipeline, &pipeline_gggui::Data {
                            vbuf, 
                            color: (self.textures[&0].1.clone(), self.sampler.clone()),
                            //scissor: current_scissor,
                            out_color: out.clone(),
                        });
                    }
                },

                Command::Textured{ texture, offset, count } => {
                    if count > 0 {
                        let (vbuf, slice) = fac.create_vertex_buffer_with_slice(&vertices[offset..offset+count], ());

                        enc.draw(&slice, &self.pipeline, &pipeline_gggui::Data {
                            vbuf, 
                            color: (self.textures[&texture].1.clone(), self.sampler.clone()),
                            //scissor: current_scissor,
                            out_color: out.clone(),
                        });
                    }
                },
            }
        }
    }
}