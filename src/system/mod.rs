// Source: https://github.com/luke-titley/imgui-docking-rs/blob/release/docking/0.5.0/imgui-examples/examples/support/mod.rs

use glium::glutin;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::{Display, Surface};
use imgui::{Context, FontConfig, FontSource, Ui, ConfigFlags};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::{rc::Rc, cell::RefCell, time::Instant};

mod clipboard;

pub struct System {
    pub first_run: bool,
    pub event_loop: Option<EventLoop<()>>,
    pub display: Rc<RefCell<glium::Display>>,
    pub imgui: Rc<RefCell<Context>>,
    pub platform: Rc<RefCell<WinitPlatform>>,
    pub renderer: Rc<RefCell<Renderer>>,
    pub font_size: f32,
}

pub fn init(title: &str) -> System {
    let title = match title.rfind('/') {
        Some(idx) => title.split_at(idx + 1).1,
        None => title,
    };
    let event_loop = EventLoop::new();
    let context = glutin::ContextBuilder::new().with_vsync(true);
    let builder = WindowBuilder::new()
        .with_title(title.to_owned())
        .with_decorations(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(1024f64, 768f64));
    let display =
        Display::new(builder, context, &event_loop)
            .expect("Failed to initialize display");

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);
    imgui.io_mut().config_flags |= ConfigFlags::DOCKING_ENABLE;

    if let Some(backend) = clipboard::init() {
        imgui.set_clipboard_backend(Box::new(backend));
    } else {
        eprintln!("Failed to initialize clipboard");
    }

    let mut platform = WinitPlatform::init(&mut imgui);
    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);
    }

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[
        FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        },
    ]);

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    let renderer = Renderer::init(&mut imgui, &display)
        .expect("Failed to initialize renderer");

    System {
        first_run: true,
        event_loop: Some(event_loop),
        display: Rc::new(RefCell::new(display)),
        imgui: Rc::new(RefCell::new(imgui)),
        platform: Rc::new(RefCell::new(platform)),
        renderer: Rc::new(RefCell::new(renderer)),
        font_size,
    }
}

impl System {
    pub fn main_loop<F: FnMut(&Self, &mut bool, &mut Ui) + 'static>(mut self, mut run_ui: F) {
        let event_loop = self.event_loop.take().unwrap();
        let display = self.display.clone();
        let imgui = self.imgui.clone();
        let platform = self.platform.clone();
        let renderer = self.renderer.clone();

        let mut last_frame = Instant::now();

        event_loop.run(move |event, _, control_flow| match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                imgui.borrow_mut().io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::MainEventsCleared => {
                let display = display.borrow();
                let gl_window = display.gl_window();

                platform
                    .borrow()
                    .prepare_frame(imgui.borrow_mut().io_mut(), &gl_window.window())
                    .expect("Failed to prepare frame");
                gl_window
                    .window()
                    .request_redraw();
            }
            Event::RedrawRequested(_) => {
                let display = display.borrow();
                let mut imgui = imgui.borrow_mut();

                let mut ui = imgui.frame();
                let mut run = true;

                run_ui(
                    &self,
                    &mut run, &mut ui
                );
                self.first_run = false;

                if !run {
                    *control_flow = ControlFlow::Exit;
                }

                let gl_window = display.gl_window();
                let mut target = display.draw();
                target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
                platform.borrow_mut().prepare_render(&ui, gl_window.window());
                let draw_data = ui.render();
                renderer
                    .borrow_mut()
                    .render(&mut target, draw_data)
                    .expect("Rendering failed");
                target.finish().expect("Failed to swap buffers");
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            event => {
                let display = display.borrow();
                let gl_window = display.gl_window();
                platform.borrow_mut().handle_event(imgui.borrow_mut().io_mut(), gl_window.window(), &event);
            }
        })
    }
}