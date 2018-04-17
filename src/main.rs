// This is a demo of RTT (render to texture) in gfx-rs.
// I don't really know why it works, so I'll leave it like this.

#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;

use gfx::traits::FactoryExt;
use gfx::Factory;
use gfx::Device;
use gfx_window_glutin as gfx_glutin;
use glutin::{GlContext, GlRequest};
use glutin::Api::OpenGl;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

const CLEAR_COLOR_A: [f32; 4] = [0.1, 0.0, 0.0, 1.0];
const CLEAR_COLOR_B: [f32; 4] = [0.0, 0.1, 0.0, 1.0];

gfx_defines!{
    vertex ColorVertex {
        pos: [f32; 2] = "i_pos",
        color: [f32; 3] = "i_color",
    }

    vertex TextureVertex {
        pos: [f32; 2] = "i_pos",
        tex_pos: [f32; 2] = "i_tex_pos",
    }

    pipeline main_pipe {
        vertex_buffer: gfx::VertexBuffer<TextureVertex> = (),
        tex: gfx::TextureSampler<[f32; 4]> = "tex_sampler",
        out: gfx::RenderTarget<ColorFormat> = "main_target",
    }

    pipeline pipe_to_texture {
        vertex_buffer: gfx::VertexBuffer<ColorVertex> = (),
        out: gfx::RenderTarget<ColorFormat> = "texture_target",
    }
}

const TRIANGLE: [ColorVertex; 3] = [
    ColorVertex {
        pos: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    },
    ColorVertex {
        pos: [0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    ColorVertex {
        pos: [0.0, 0.5],
        color: [0.0, 0.0, 1.0],
    },
];

const RECTANGLE: [TextureVertex; 6] = [
    TextureVertex {
        pos: [-0.5, 0.5],
        tex_pos: [0.0, 1.0],
    },
    TextureVertex {
        pos: [0.5, 0.5],
        tex_pos: [1.0, 1.0],
    },
    TextureVertex {
        pos: [0.5, -0.5],
        tex_pos: [1.0, 0.0],
    },
    TextureVertex {
        pos: [-0.5, 0.5],
        tex_pos: [0.0, 1.0],
    },
    TextureVertex {
        pos: [0.5, -0.5],
        tex_pos: [1.0, 0.0],
    },
    TextureVertex {
        pos: [-0.5, -0.5],
        tex_pos: [0.0, 0.0],
    },
];

pub fn main() {
    const W: u32 = 512;
    const H: u32 = 512;

    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_title("RTT demo".to_string())
        .with_dimensions(W, H);
    let context_builder = glutin::ContextBuilder::new()
        .with_gl(GlRequest::Specific(OpenGl, (3, 2)))
        .with_vsync(true);
    let (window, mut device, mut factory, rtv, _) =
        gfx_glutin::init::<ColorFormat, DepthFormat>(window_builder, context_builder, &events_loop);
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let vs = br#"
        #version 150 core

        in vec2 i_pos;
        in vec2 i_tex_pos;

        out vec2 v_tex_pos;

        void main() {
            v_tex_pos = i_tex_pos;
            gl_Position = vec4(i_pos, 0.0, 1.0);
        }
    "#;

    let fs = br#"
        #version 150 core

        in vec2 v_tex_pos;
        out vec4 f_color;

        uniform sampler2D tex_sampler;

        void main() {
            f_color = texture(tex_sampler, v_tex_pos);
        }
    "#;

    let main_pso = factory
        .create_pipeline_simple(&vs[..], &fs[..], main_pipe::new())
        .unwrap();

    let vs = br#"
        #version 150 core

        in vec2 i_pos;
        in vec3 i_color;

        out vec3 v_color;

        void main() {
            v_color = i_color;
            gl_Position = vec4(i_pos, 0.0, 1.0);
        }
    "#;

    let fs = br#"
        #version 150 core

        in vec3 v_color;
        out vec4 f_color;

        void main() {
            f_color = vec4(v_color, 1.0);
        }
    "#;

    let texture_pso = factory
        .create_pipeline_simple(&vs[..], &fs[..], pipe_to_texture::new())
        .unwrap();

    let (triangle_buf, triangle_slice) = factory.create_vertex_buffer_with_slice(&TRIANGLE, ());
    let (rect_buf, rect_slice) = factory.create_vertex_buffer_with_slice(&RECTANGLE, ());

    let sampler = factory.create_sampler_linear();
    let (_, srv, texture_out) = factory.create_render_target(W as u16, H as u16).unwrap();

    // used to draw the texture on screen
    let main_uniforms = main_pipe::Data {
        vertex_buffer: rect_buf.clone(),
        tex: (srv, sampler), // actual texture sampler
        out: rtv,
    };

    // used to draw a triangle on the texture
    let texture_uniforms = pipe_to_texture::Data {
        vertex_buffer: triangle_buf,
        out: texture_out,
    };

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent { event, .. } = event {
                match event {
                    glutin::WindowEvent::Closed
                    | glutin::WindowEvent::KeyboardInput {
                        input:
                            glutin::KeyboardInput {
                                virtual_keycode: Some(glutin::VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => running = false,
                    _ => {}
                }
            }
        });

        // triangle
        encoder.clear(&texture_uniforms.out, CLEAR_COLOR_B);
        encoder.draw(&triangle_slice, &texture_pso, &texture_uniforms);

        // texture
        encoder.clear(&main_uniforms.out, CLEAR_COLOR_A);
        encoder.draw(&rect_slice, &main_pso, &main_uniforms);

        encoder.flush(&mut device);

        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
