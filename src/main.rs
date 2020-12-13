pub mod bstroke;
pub mod geometry;

pub const BENCH_STROKES_NUM: usize = 10;

use miniquad::*;
use glam::vec2;

use bezier::Timer;

use bstroke::Strokes;

#[cfg(target_arch="x86_64")]
pub const TEXTURE_WIDTH: u32 = 3840;
#[cfg(target_arch="x86_64")]
pub const TEXTURE_HEIGHT: u32 = 2160;

struct Stage {
    strokes: Strokes,
    offscreen_pass: RenderPass,
    _timer: Timer,
}

impl Stage {
    pub fn new(ctx: &mut Context, max_curves_num: usize) -> Stage {
        let color_img = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: TEXTURE_WIDTH,
                height: TEXTURE_HEIGHT,
                format: TextureFormat::RGBA8,
                ..Default::default()
            },
        );
        let depth_img = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: TEXTURE_WIDTH,
                height: TEXTURE_HEIGHT,
                format: TextureFormat::Depth,
                ..Default::default()
            },
        );
        let offscreen_pass = RenderPass::new(ctx, color_img, depth_img);
        Stage {
            strokes: Strokes::new(ctx, max_curves_num),
            offscreen_pass,
            _timer: Timer::new(100),
        }
    }
}

impl EventHandler for Stage {
    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        self.strokes.mouse_button_down_event(ctx, button, x, y);
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        self.strokes.mouse_motion_event(ctx, x, y);
    }

    fn update(&mut self, ctx: &mut Context) {
        self.strokes.update(ctx);
    }

    fn draw(&mut self, ctx: &mut Context) {
       ctx.begin_default_pass(
           PassAction::clear_color(0.5, 0.5, 0.5, 0.),
       );
       self.strokes.draw(ctx);
       ctx.end_render_pass();
        ctx.commit_frame();
    }
}

fn main() {
    miniquad::start(conf::Conf::default(), |mut ctx| {
        UserData::owning(Stage::new(&mut ctx, 1000), ctx)
    });
}
