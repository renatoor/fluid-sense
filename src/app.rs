use crate::gfx::renderer::Renderer;

use std::time::{Duration, Instant};
use winit::dpi::{PhysicalPosition};
use winit::platform::unix::WindowBuilderExtUnix;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub trait App: 'static + Sized {
    fn init(renderer: &mut Renderer) -> Self;
    fn keyboard_input(&mut self, input: KeyboardInput);
    fn mouse_movement(&mut self, dx: f32, dy: f32);
    fn update(&mut self, dt: Duration);
    fn resize(&mut self, width: u32, height: u32);
    fn render(&self, renderer: &Renderer) -> Result<(), wgpu::SurfaceError>;
    fn render2<'a>(&'a mut self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass<'a>);
}

pub async fn run<A: App>() {
    env_logger::init();

    let window_instance = "fluid-sense".to_string();
    let window_class = "fluid-sense".to_string();

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_class(window_instance, window_class)
        .build(&event_loop)
        .unwrap();

    let mut renderer = Renderer::new(&window).await;

    let mut app = A::init(&mut renderer);
    let mut last_update = Instant::now();

    let size = window.inner_size();
    let window_center = PhysicalPosition::new((size.width / 2) as i32, (size.height / 2) as i32);

    window.set_cursor_position(window_center).unwrap();
    window.set_cursor_grab(true).unwrap();
    window.set_cursor_visible(false);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => {
                app.resize((*physical_size).width, (*physical_size).height);
                renderer.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                app.resize((**new_inner_size).width, (**new_inner_size).height);
                renderer.resize(**new_inner_size);
            }
            _ => {}
        },
        Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { delta: (dx, dy) },
            ..
        } => {
            app.mouse_movement(dx as f32, dy as f32);

            window.set_cursor_position(window_center).unwrap();
        }
        Event::DeviceEvent {
            event: DeviceEvent::Key(input),
            ..
        } => {
            app.keyboard_input(input);
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            let now = Instant::now();
            let dt = now - last_update;

            last_update = now;

            app.update(dt);

            //match app.render(&renderer) {
            match renderer.render_fn(&mut app) {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => renderer.configure_surface(),
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    });
}
