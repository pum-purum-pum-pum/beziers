use glam::{vec2, Vec2};
use miniquad::*;
use std::time::Duration;

use crate::Timer;

use crate::geometry::*;

pub const BENCH_STROKES_NUM: usize = 100;

pub struct Strokes {
    pipeline: Pipeline,
    bindings: Bindings,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    path_width: f32,
    path: BezierPath,

    timer: Timer,
}

impl Strokes {
    pub fn new(ctx: &mut Context, max_curves_num: usize) -> Strokes {
        let vertex_buffer = Buffer::stream(
            ctx,
            BufferType::VertexBuffer,
            max_curves_num * std::mem::size_of::<Vertex>(),
        );

        let index_buffer = Buffer::stream(
            ctx,
            BufferType::IndexBuffer,
            max_curves_num * std::mem::size_of::<u16>(),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: vec![],
        };
        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::meta()).unwrap();

        let pipeline = Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("a", VertexFormat::Float2),
                VertexAttribute::new("control", VertexFormat::Float2),
                VertexAttribute::new("c", VertexFormat::Float2),
                VertexAttribute::new("thickness", VertexFormat::Float1),
            ],
            shader,
            PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        );
        let path = BezierPath::default();

        let mut stage = Strokes {
            pipeline,
            bindings,
            indices: Vec::with_capacity(max_curves_num * 6),
            vertices: Vec::with_capacity(max_curves_num * 4),
            vertex_buffer,
            index_buffer,
            path,
            path_width: 10.,
            timer: Timer::new(100),
        };
        stage.update_buffers(ctx);
        stage
    }

    pub fn update_buffers(&mut self, ctx: &mut Context) {
        let (vertices, indices) = self.path.vertices(self.path_width);
        self.vertices = vertices;
        self.indices = indices;
        self.vertex_buffer.update(ctx, &self.vertices);
        self.index_buffer.update(ctx, &self.indices);
    }
}

#[derive(Debug, Default)]
pub struct Shape {
    pub regular: Vec<Vec2>,
    pub holes: Vec<Vec<Vec2>>,
}

impl Shape {
    pub fn from_regular(regular: Vec<Vec2>) -> Shape {
        Shape {
            regular,
            holes: Vec::new(),
        }
    }
}

impl EventHandler for Strokes {
    fn mouse_button_down_event(&mut self, ctx: &mut Context, _button: MouseButton, x: f32, y: f32) {
        self.path.stroke(vec2(x, y));
        self.update_buffers(ctx);
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        let mut to_undo = false;
        if self.path.control.is_some() {
            to_undo = true;
            self.path.stroke(vec2(x, y));
        }

        self.update_buffers(ctx);
        if to_undo {
            self.path.undo();
        }
    }

    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        let (w, h) = ctx.screen_size();
        let size = vec2(w, h);
        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_uniforms(&size);
        ctx.apply_bindings(&self.bindings);
        ctx.draw(0, self.indices.len() as i32, 1);
        if let Some(avg) = self.timer.tick() {
            #[cfg(not(target_arch = "wasm32"))]
            println!("{:?} fps", Duration::new(1, 0).as_nanos() / avg.as_nanos());
        }
    }
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"# version 100
    uniform vec2 resolution;
    attribute vec2 pos;
    attribute vec2 a;
    attribute vec2 control;
    attribute vec2 c;
    attribute float thickness;

    varying vec2 af;
    varying vec2 controlf;
    varying vec2 cf;
    varying vec2 posf;
    varying float thicknessf;

    void main() {
        vec2 ps = vec2(2.* pos.x / resolution.x - 1., -2. * pos.y / resolution.y + 1.);
        af = a;
        controlf = control;
        cf = c;
        posf = pos;
        thicknessf = thickness;
        gl_Position = vec4(ps, 0., 1.);
    }"#;

    pub const FRAGMENT: &str = r#"# version 100
    precision highp float;
    varying vec2 af;
    varying vec2 controlf;
    varying vec2 cf;
    varying vec2 posf;
    varying float thicknessf;

    
    float dot2( in vec2 v ) { return dot(v,v); }

    float sdBezier( in vec2 pos, in vec2 A, in vec2 B, in vec2 C )
    {    
        vec2 a = B - A;
        vec2 b = A - 2.0*B + C;
        vec2 c = a * 2.0;
        vec2 d = A - pos;
        float kk = 1.0/dot(b,b);
        float kx = kk * dot(a,b);
        float ky = kk * (2.0*dot(a,a)+dot(d,b)) / 3.0;
        float kz = kk * dot(d,a);      
        float res = 0.0;
        float p = ky - kx*kx;
        float p3 = p*p*p;
        float q = kx*(2.0*kx*kx-3.0*ky) + kz;
        float h = q*q + 4.0*p3;
        if( h >= 0.0) 
        { 
            h = sqrt(h);
            vec2 x = (vec2(h,-h)-q)/2.0;
            vec2 uv = sign(x)*pow(abs(x), vec2(1.0/3.0));
            float t = clamp( uv.x+uv.y-kx, 0.0, 1.0 );
            res = dot2(d + (c + b*t)*t);
        }
        else
        {
            float z = sqrt(-p);
            float v = acos( q/(p*z*2.0) ) / 3.0;
            float m = cos(v);
            float n = sin(v)*1.732050808;
            vec3  t = clamp(vec3(m+m,-n-m,n-m)*z-kx,0.0,1.0);
            res = min( dot2(d+(c+b*t.x)*t.x),
                       dot2(d+(c+b*t.y)*t.y) );
            // the third root cannot be the closest
            // res = min(res,dot2(d+(c+b*t.z)*t.z));
        }
        return sqrt( res );
    }

    void main() {
        vec4 color = vec4(1., 1., 1., 1.);
        float d = sdBezier(posf, af, controlf, cf) - thicknessf;
        float s = smoothstep(0., 1., -d);
        if (d < 0.) {
            color.a = s;
        } else {
            discard;
        }
        gl_FragColor = color;
    }"#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("resolution", UniformType::Float2)],
            }
        }
    }
}
