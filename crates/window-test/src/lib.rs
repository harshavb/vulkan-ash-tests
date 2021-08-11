use cgmath::{Matrix4, Vector4};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn run() {
    // Type which runs a loop for handling... well, everything
    let event_loop = EventLoop::new();

    // WindowBuilder allows custom window settings to be set before being built
    let mut builder = WindowBuilder::new();
    builder = builder.with_inner_size(LogicalSize::new(400, 400));
    let window = builder.build(&event_loop).unwrap();

    let vec = Vector4::new(4.0, 5.0, 6.0, 7.0);
    let matrix = Matrix4::from_cols(vec, vec.yxzy(), vec.zxxz(), vec.xzxy());
    let test = matrix * vec;
    println!("{:?}", test);

    event_loop.run(move |event, _, control_flow| {
        // ControlFlow::Poll continually runs the event loop
        *control_flow = ControlFlow::Poll;

        // ControlFlow::Wait pauses the loop if nothing is available to process
        // *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Close button was pressed");
                *control_flow = ControlFlow::Exit;
            }
            // Updates application
            Event::MainEventsCleared => {
                // If redrawing continuously, renders here
                // If redraw is only occasional, calls request_redraw()
                window.request_redraw();
            }
            // Redraws application - only used for applications that do not render continuously
            Event::RedrawRequested(_) => {}
            _ => (),
        }
    })
}
