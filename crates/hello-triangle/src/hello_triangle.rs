use app::graphics::vulkan_base::{VulkanBase, WindowDimensions};
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
    pub fn new() -> (Self, EventLoop<()>) {
        let event_loop = EventLoop::new();

        // Stores window dimensions
        let width = 400;
        let height = 400;

        // Creates a window using a WindowBuilder
        let mut builder = WindowBuilder::new();
        builder = builder
            .with_title("name of window")
            .with_inner_size(LogicalSize::new(width, height));
        let _window = builder
            .build(&event_loop)
            .expect("Could not create a window!");

        // Stores window information for use in VulkanBase
        let window_dimensions = WindowDimensions::new(width, height);

        // Creates a VulkanType holding all the vulkan data
        let _vulkan_type = VulkanBase::new(&_window, &window_dimensions);

        let app = TriangleApplication {
            _window,
            _vulkan_type,
        };

        (app, event_loop)
    }

    fn example_function(&self) {}
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
                app.example_function(); // TEST FUNCTION CALL - TO BE REMOVED
                println!("Close button was pressed");
                *control_flow = ControlFlow::Exit;
            }
            // Updates application
            Event::MainEventsCleared => {}
            _ => (),
        }
    });
}
