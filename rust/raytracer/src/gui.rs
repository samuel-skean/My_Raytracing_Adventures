use pixels::{Pixels, SurfaceTexture};
use winit::{application::ApplicationHandler, dpi::LogicalSize, event::WindowEvent, event_loop::EventLoop, platform::macos::WindowAttributesExtMacOS, window::{Window, WindowAttributes}};

use crate::{PixelGrid, Resolution};

struct App<'a> {
    window: Option<Window>,
    pixels: Option<Pixels>,
    res: Resolution,
    image: &'a PixelGrid,
}

impl<'a> App<'a> {
    fn new(res: Resolution, image: &'a PixelGrid) -> Self {
        App {
            window: None,
            pixels: None,
            res,
            image
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let size = LogicalSize::new(self.res.width as f64, self.res.height as f64);
        let window_attributes = WindowAttributes::default()
            .with_title(env!("CARGO_BIN_NAME"))
            .with_inner_size(size)
            .with_movable_by_window_background(true);
        let window = event_loop.create_window(window_attributes).unwrap();
        let pixels = {
            let window_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(self.res.width as u32, self.res.height as u32, surface_texture).unwrap()
        };
        window.request_redraw();
        self.window = Some(window);
        self.pixels = Some(pixels);
        
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                // TODO: Fix this control flow code. I think I could really do with a C++ jthread right about now...
                // std::process::exit(0);
            },
            WindowEvent::RedrawRequested => {
                render(&self.image, &mut self.pixels.as_mut().unwrap());
            },
            WindowEvent::Resized(size) => {
                self.pixels.as_mut().unwrap().resize_surface(size.width, size.height).unwrap();
            }
            _ => {},
        }
        self.window.as_mut().unwrap().request_redraw();

    }
}

pub fn run_gui(image: &PixelGrid, res: Resolution) {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(res, image);

    event_loop.run_app(&mut app).unwrap();
}

#[inline]
fn render(image: &PixelGrid, pixels: &mut Pixels) {
    let slice_of_colors: &mut [[u8; 4]] = bytemuck::cast_slice_mut(pixels.frame_mut());
    for (i, p) in image.iter().flatten().enumerate() {
        let pixel_info = p.load(atomic::Ordering::Acquire);
        slice_of_colors[i] = pixel_info.accumulated_color.for_winit(pixel_info.samples_so_far);
    }
    pixels.render().unwrap();
}