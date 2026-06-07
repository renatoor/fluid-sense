use crate::gfx::renderer::{RenderStatus, Renderer};

use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{ElementState, KeyEvent, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{UserAttentionType, Window, WindowId};

#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};

pub trait App: 'static + Sized {
    fn init(renderer: &mut Renderer) -> Self;
    fn keyboard_input(&mut self, input: &KeyEvent);
    fn mouse_movement(&mut self, dx: f32, dy: f32);
    fn update(&mut self, dt: Duration);
    fn resize(&mut self, width: u32, height: u32);
    fn render<'a>(&'a mut self, renderer: &Renderer, render_pass: &mut wgpu::RenderPass<'a>);
}

struct Runner<A: App> {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    app: Option<A>,
    last_update: Instant,
    logged_first_frame: bool,
    logged_event_loop: bool,
    last_cursor_position: Option<(f64, f64)>,
    _marker: PhantomData<A>,
}

impl<A: App> Default for Runner<A> {
    fn default() -> Self {
        Self {
            window: None,
            renderer: None,
            app: None,
            last_update: Instant::now(),
            logged_first_frame: false,
            logged_event_loop: false,
            last_cursor_position: None,
            _marker: PhantomData,
        }
    }
}

impl<A: App> Runner<A> {
    fn window_id(&self) -> Option<WindowId> {
        self.window.as_ref().map(|window| window.id())
    }

    fn request_redraw(&self) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn handle_resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        if let Some(app) = self.app.as_mut() {
            app.resize(width, height);
        }

        if let Some(renderer) = self.renderer.as_mut() {
            renderer.resize((width, height).into());
        }
    }
}

impl<A: App> ApplicationHandler for Runner<A> {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init && !self.logged_event_loop {
            eprintln!("fluid-sense: event loop started");
            self.logged_event_loop = true;
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        eprintln!("fluid-sense: creating window");
        let attrs = Window::default_attributes()
            .with_title("fluid-sense")
            .with_inner_size(LogicalSize::new(1280.0, 720.0))
            .with_position(LogicalPosition::new(80.0, 80.0))
            .with_visible(false);
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        eprintln!("fluid-sense: window created");

        let mut renderer = pollster::block_on(Renderer::new(Arc::clone(&window)));
        eprintln!("fluid-sense: renderer initialized");

        let app = A::init(&mut renderer);
        eprintln!("fluid-sense: app initialized");

        window.set_visible(true);
        window.request_user_attention(Some(UserAttentionType::Critical));
        window.request_redraw();
        eprintln!("fluid-sense: window shown");

        self.last_update = Instant::now();
        self.window = Some(window);
        self.renderer = Some(renderer);
        self.app = Some(app);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if Some(window_id) != self.window_id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed
                    && event.physical_key == PhysicalKey::Code(KeyCode::Escape)
                {
                    event_loop.exit();
                    return;
                }

                if let Some(app) = self.app.as_mut() {
                    app.keyboard_input(&event);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some((last_x, last_y)) = self.last_cursor_position {
                    if let Some(app) = self.app.as_mut() {
                        app.mouse_movement(
                            (position.x - last_x) as f32,
                            (position.y - last_y) as f32,
                        );
                    }
                }
                self.last_cursor_position = Some((position.x, position.y));
            }
            WindowEvent::CursorLeft { .. } | WindowEvent::Focused(false) => {
                self.last_cursor_position = None;
            }
            WindowEvent::Resized(physical_size) => {
                self.handle_resize(physical_size.width, physical_size.height);
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(window) = &self.window {
                    let size = window.inner_size();
                    self.handle_resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now - self.last_update;
                self.last_update = now;

                let Some(app) = self.app.as_mut() else {
                    return;
                };
                let Some(renderer) = self.renderer.as_mut() else {
                    return;
                };

                app.update(dt);

                match renderer.render(app) {
                    RenderStatus::Rendered => {
                        if !self.logged_first_frame {
                            eprintln!(
                                "fluid-sense window is rendering at {}x{}",
                                renderer.get_size().0,
                                renderer.get_size().1
                            );
                            self.logged_first_frame = true;
                        }
                    }
                    RenderStatus::Reconfigure => renderer.configure_surface(),
                    RenderStatus::Skip => {}
                    RenderStatus::Failed(message) => eprintln!("{}", message),
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.request_redraw();
    }
}

pub fn run<A: App>() {
    eprintln!("fluid-sense: starting gui");
    env_logger::init();

    eprintln!("fluid-sense: creating event loop");
    let mut event_loop_builder = EventLoop::builder();

    #[cfg(target_os = "macos")]
    event_loop_builder
        .with_activation_policy(ActivationPolicy::Regular)
        .with_activate_ignoring_other_apps(true);

    let event_loop = event_loop_builder.build().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    eprintln!("fluid-sense: event loop created");

    let mut runner = Runner::<A>::default();
    event_loop.run_app(&mut runner).unwrap();
}
