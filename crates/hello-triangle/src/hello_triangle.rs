use app::graphics::vulkan_initializer::VulkanBase;
use std::error::Error;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

pub struct TriangleApplication {
    _window: Window,
    _vulkan_type: VulkanBase,
}

impl TriangleApplication {
    // Creates the window and event loop for the application
    pub fn new() -> Result<(Self, EventLoop<()>), Box<dyn Error>> {
        let event_loop = EventLoop::new();

        // Creates a window using a WindowBuilder
        let mut builder = WindowBuilder::new();
        builder = builder
            .with_inner_size(LogicalSize::new(400, 400))
            .with_title("name of window");
        let _window = builder.build(&event_loop)?;

        // Creates a VulkanType holding all the vulkan data
        let _vulkan_type = VulkanBase::new(&_window)?;

        let app = TriangleApplication {
            _window,
            _vulkan_type,
        };

        Ok((app, event_loop))
    }

    fn init_vulkan(&self) {}
}

impl Drop for TriangleApplication {
    fn drop(&mut self) {
        println!("Cleaning up TriangleApplication!");
        println!("Cleaned up TriangleApplication!");
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
                app.init_vulkan(); // TEST FUNCTION CALL - TO BE REMOVED
                println!("Close button was pressed");
                *control_flow = ControlFlow::Exit;
            }
            // Updates application
            Event::MainEventsCleared => {}
            _ => (),
        }
    });
}
