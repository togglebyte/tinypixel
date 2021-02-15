use futures::executor::block_on;
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event::Event as WinitEvent,
    event::*,
    event_loop::{ControlFlow, EventLoop as WinitEventLoop},
    window::{Window, WindowBuilder},
};

use crate::ScreenSize;
use crate::renderer::Renderer;

pub enum Event<'a> {
    Key(&'a KeyboardInput),
    Mouse,
}

pub trait EventLoop: 'static {
    fn draw(&mut self, renderer: &mut Renderer);
    fn update(&mut self);
    fn resize(&mut self, new_size: ScreenSize);
    fn input<'a>(&mut self, event: Event<'a>);
}

pub fn start<T: std::fmt::Debug>(mut el: impl EventLoop, window: Window, event_loop: WinitEventLoop<T>) {
    let mut renderer = Renderer::new(
        window.inner_size().width,
        window.inner_size().height,
        &window,
    );

    event_loop.run(move |event, _, control_flow| {
        match event {
            WinitEvent::RedrawRequested(window_id) if window_id == window.id() => {
                el.draw(&mut renderer);
            }
            WinitEvent::MainEventsCleared => {
                el.update();
                renderer.render();
                window.request_redraw();
            }
            WinitEvent::WindowEvent { ref event, window_id, .. } if window_id == window.id() => {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        el.resize(ScreenSize::new(physical_size.width, physical_size.height));
                        renderer.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        el.resize(ScreenSize::new(new_inner_size.width, new_inner_size.height));
                        renderer.resize(**new_inner_size)
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        el.input(Event::Key(input));

                        match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => {
                                // quit
                                *control_flow = ControlFlow::Exit;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ => { }
        }
    });
}
