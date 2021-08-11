use std::error::Error;
use winit::{
    dpi::LogicalSize,
    event_loop::{EventLoop},
    window::{Window, WindowBuilder},
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

pub struct TriangleApplication {
    _window: Window,
}

impl TriangleApplication {
    // Creates the window and event loop for the application
    pub fn new() -> Result<(Self, EventLoop<()>), Box<dyn Error>> {
        let event_loop = EventLoop::new();

        let mut builder = WindowBuilder::new();
        builder = builder.with_inner_size(LogicalSize::new(400, 400));
        let _window = builder.build(&event_loop)?;

        let app = TriangleApplication { _window };
        app.init_vulkan();

        Ok((app, event_loop))
    }

    fn init_vulkan(&self) {}
}

impl Drop for TriangleApplication {
    fn drop(&mut self) {
        println!("graceful cleanup");
    }
}

pub fn run(app: TriangleApplication, event_loop: EventLoop<()>) {
    event_loop.run(move |event, _, control_flow| {
        // Continually runs the event loop
        *control_flow = ControlFlow::Poll;

        match event {
            // Checks for close requested
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                app.init_vulkan();
                println!("Close button was pressed");
                *control_flow = ControlFlow::Exit;
            }
            // Updates application
            Event::MainEventsCleared => {}
            _ => (),
        }
    });
}
