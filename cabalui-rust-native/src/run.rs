use std::ffi::CString;
use gl::types::*;
use glfw::{Action, Context, Key, PWindow};
use skia_safe::gpu::{backend_render_targets, direct_contexts, DirectContext, SurfaceOrigin, surfaces};
use skia_safe::gpu::gl::{Format, FramebufferInfo, Interface};
use skia_safe::{Color, ColorType, Paint, Rect, scalar, Surface};

struct Env {
    surface: Surface,
    gr_context: DirectContext,
    window: PWindow,
    fb_info: FramebufferInfo
}

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    let (mut window, events) = glfw.create_window(800, 600, "Hello this is window", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.make_current();
    
    gl::load_with(|s| {
       window.get_proc_address(s) 
    });
    
    let interface = Interface::new_load_with(|name| {
       window.get_proc_address(name) 
    }).expect("Couldn't create interface");
    
    let mut gr_context = direct_contexts::make_gl(interface, None)
        .expect("Couldn't create direct context");
    
    let fb_info = {
        let mut fbiod: GLint = 0;
        unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fbiod)}
        
        FramebufferInfo {
            fboid: fbiod.try_into().unwrap(),
            format: Format::RGBA8.into(),
            ..Default::default()
        }
    };
    
    fn create_surface(
        window: &PWindow,
        fb_info: FramebufferInfo,
        gr_context: &mut DirectContext
    ) -> Surface {
        let size = window.get_framebuffer_size();
        
        let backend_render_target = 
            backend_render_targets::make_gl(size, 0, 0, fb_info);
        
        surfaces::wrap_backend_render_target(
            gr_context,
            &backend_render_target,
            SurfaceOrigin::BottomLeft,
            ColorType::RGBA8888,
            None,
            None
        ).expect("Couldn't create skia surface")
    }
    
    let surface = create_surface(&window, fb_info, &mut gr_context);

    let mut env = Env {
        surface,
        gr_context,
        window,
        fb_info
    };

    fn handle_window_event(env: &mut Env, event: glfw::WindowEvent, ) {
        match event {
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                env.window.set_should_close(true)
            }
            glfw::WindowEvent::FramebufferSize(w, h) => {
                env.surface = create_surface(&env.window, env.fb_info, &mut env.gr_context);
            }
            _ => {}
        }
    }
    
    env.window.set_framebuffer_size_polling(true);

    while !env.window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            handle_window_event(&mut env, event);
        }
        
        let canvas = env.surface.canvas();
        canvas.clear(Color::WHITE);
        
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_stroke_width(4.0);
        paint.set_color(Color::from_rgb(255, 100, 0));
        
        let rect = Rect::new(10.0, 10.0, 100.0, 160.0);
        canvas.draw_rect(rect, &paint);
        
        env.gr_context.flush_and_submit();
        env.window.swap_buffers();
    }
}