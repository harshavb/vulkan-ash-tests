use std::error::Error;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct TriangleApplication {
    event_loop: Option<EventLoop<()>>,
    _window: Window,
}

impl TriangleApplication {
    // Runs the application
    pub fn run() -> Result<String, Box<dyn Error>> {
        let mut application = TriangleApplication::new()?;
        application.init_vulkan();
        application.main_loop();
        Ok(String::from("Successfully Ran"))
    }

    // Creates the window and event loop for the application
    fn new() -> Result<Self, Box<dyn Error>> {
        let event_loop = EventLoop::new();

        let mut builder = WindowBuilder::new();
        builder = builder.with_inner_size(LogicalSize::new(400, 400));
        let _window = builder.build(&event_loop)?;

        Ok(TriangleApplication {
            event_loop: Some(event_loop),
            _window,
        })
    }

    fn init_vulkan(&self) {}

    fn main_loop(&mut self) {
        self.event_loop.take().unwrap().run(move |event, _, control_flow| {
            // Continually runs the event loop
            *control_flow = ControlFlow::Poll;

            match event {
                // Checks for close requested
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    println!("Close button was pressed");
                    *control_flow = ControlFlow::Exit;
                }
                // Updates application
                Event::MainEventsCleared => {}
                _ => (),
            }
        })
    }
}

impl Drop for TriangleApplication {
    fn drop(&mut self) {
        println!("graceful cleanup");
    }
}
