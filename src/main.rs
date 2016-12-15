#![feature(conservative_impl_trait)]

extern crate env_logger;
extern crate getopts;
extern crate time;
extern crate glutin;
extern crate rand;
extern crate vecmath;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
#[cfg(target_os = "windows")]
extern crate gfx_device_dx11;
extern crate gfx_window_glutin;
//extern crate gfx_window_glfw;
#[cfg(target_os = "windows")]
extern crate gfx_window_dxgi;

pub use app::ColorFormat;
use gfx::{Bundle, texture};
use std::thread;
use std::time::Duration;

pub mod app;
pub mod shade;

pub type TexFormat = [f32; 4];

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_TexCoord",
    }

    constant Locals {
        inv_size: [f32; 2] = "u_InvSize",
        init: f32 = "u_Init",
    }

    pipeline gol {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        src: gfx::TextureSampler<[f32; 4]> = "t_Src",
        dest: gfx::RenderTarget<[f32; 4]> = "Target0",
    }

    pipeline display {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        src: gfx::TextureSampler<[f32; 4]> = "t_Src",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

impl Vertex {
    fn new(p: [f32; 2], uv: [f32; 2]) -> Vertex {
        Vertex {
            pos: p,
            uv: uv,
        }
    }
}

struct ViewPair<R: gfx::Resources, T: gfx::format::TextureFormat + gfx::format::RenderFormat> {
    resource: gfx::handle::ShaderResourceView<R, T::View>,
    target: gfx::handle::RenderTargetView<R, T>,
}

impl<R: gfx::Resources, T: gfx::format::TextureFormat + gfx::format::RenderFormat> ViewPair<R, T> {
    fn new<F: gfx::Factory<R>>(factory: &mut F, width: u16, height: u16) -> Self {
        let (_ , srv, rtv) = factory.create_render_target(width, height).expect("Failed to create render target");
        ViewPair { resource: srv, target: rtv }
    }
}

struct App<R: gfx::Resources> {
    init: bool,
    size: [u16; 2],
    gol: Bundle<R, gol::Data<R>>,
    display: Bundle<R, display::Data<R>>,
    images: [ViewPair<R, [f32; 4]>; 2],
}

impl<R: gfx::Resources> app::Application<R> for App<R> {
    fn new<F: gfx::Factory<R>>(mut factory: F, init: app::Init<R>) -> Self {
        use gfx::traits::FactoryExt;

        let (width, height, _, _) = init.color.get_dimensions();

        let vs = shade::Source {
            hlsl_40:  include_bytes!("../data/vs.fx"),
            .. shade::Source::empty()
        };
        let ps = shade::Source {
            hlsl_40:  include_bytes!("../data/ps.fx"),
            .. shade::Source::empty()
        };
        let ps_display = shade::Source {
            hlsl_40:  include_bytes!("../data/ps_display.fx"),
            .. shade::Source::empty()
        };
        let sampler = factory.create_sampler(
            texture::SamplerInfo::new(texture::FilterMethod::Scale, texture::WrapMode::Clamp)
        );
        let vertex_data = [
            Vertex::new([-1.0, -1.0], [0.0, 1.0]),
            Vertex::new([1.0, -1.0], [1.0, 1.0]),
            Vertex::new([-1.0, 1.0], [0.0, 0.0]),
            Vertex::new([-1.0, 1.0], [0.0, 0.0]),
            Vertex::new([1.0, -1.0], [1.0, 1.0]),
            Vertex::new([1.0, 1.0], [1.0, 0.0]),
        ];

        let (vbuf, slice) = factory.create_vertex_buffer_with_slice(&vertex_data, ());

        let gol_pso = factory.create_pipeline_simple(
            vs.select(init.backend).unwrap(),
            ps.select(init.backend).unwrap(),
            gol::new()
        ).unwrap();

        let display_pso = factory.create_pipeline_simple(
            vs.select(init.backend).unwrap(),
            ps_display.select(init.backend).unwrap(),
            display::new()
        ).unwrap();

        let images = [
            ViewPair::new(&mut factory, width, height),
            ViewPair::new(&mut factory, width, height),
        ];

        let cbuf = factory.create_constant_buffer(1);

        let gol_data = gol::Data {
            vbuf: vbuf.clone(),
            locals: cbuf,
            dest: images[0].target.clone(),
            src: (images[1].resource.clone(), sampler.clone()),
        };

        let display_data = display::Data {
            vbuf: vbuf.clone(),
            src: (images[0].resource.clone(), sampler.clone()),
            out: init.color,
        };

        App {
            init: true,
            size: [width, height],
            gol: Bundle::new(slice.clone(), gol_pso, gol_data),
            display: Bundle::new(slice.clone(), display_pso, display_data),
            images: images,
        }
    }

    fn render<C: gfx::CommandBuffer<R>>(&mut self, encoder: &mut gfx::Encoder<R, C>) {
        self.images.swap(0, 1);
        self.gol.data.src.0 = self.images[0].resource.clone();
        self.display.data.src.0 = self.images[1].resource.clone();
        self.gol.data.dest = self.images[1].target.clone();

        let locals = Locals { init: if self.init { 1.0 } else { 0.0 }, inv_size: [1.0/self.size[0] as f32, 1.0/self.size[1] as f32] };
        self.init = false;
        encoder.clear(&self.gol.data.dest, [0.0, 0.0, 0.0, 1.0]);
        encoder.update_constant_buffer(&self.gol.data.locals, &locals);
        self.gol.encode(encoder);
        encoder.clear(&self.display.data.out, [1.0, 0.0, 0.0, 1.0]);
        self.display.encode(encoder);
        thread::sleep(Duration::from_millis(1));
    }
}

fn main() {
    use app::Application;
    App::launch_default("Game-of-life simulation with gfx-rs");
}
