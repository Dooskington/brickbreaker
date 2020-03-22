use crate::{input::InputState, renderer::Renderer};
use ::winit::{
    dpi::LogicalSize,
    event::Event as WinitEvent,
    event::WindowEvent as WinitWindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use time::{Duration, Instant};

pub use ::winit::window::Window as WinitWindow;

const SIXTY_FPS_DT: f64 = 1.0 / 60.0;

pub fn run<T>(
    title: &str,
    width: u32,
    height: u32,
    state: T,
    init_callback: impl FnMut(&mut Renderer, &mut T) + 'static,
    tick_callback: impl FnMut(&mut T) + 'static,
    render_callback: impl FnMut(u128, &mut Renderer, &T) + 'static,
) where
    T: 'static,
{
    let event_loop = EventLoop::new();
    let window: WinitWindow = WindowBuilder::new()
        .with_title(title)
        .with_min_inner_size(LogicalSize::new(width, height))
        .with_inner_size(LogicalSize::new(width, height))
        .build(&event_loop)
        .expect("Failed to create window!");

    let mut init_callback = Box::new(init_callback);
    let mut tick_callback = Box::new(tick_callback);
    let mut render_callback = Box::new(render_callback);

    let mut renderer: Renderer = Renderer::new(&window);
    let mut state: T = state;

    let one_second: Duration = Duration::seconds(1);
    let mut fps_timer: Duration = Duration::zero();
    let mut fps_counter: u32 = 0;
    let mut fps: u32 = 0;

    let target_dt: f64 = SIXTY_FPS_DT;
    let mut time: f64 = 0.0;
    let mut current_time = Instant::now();
    let mut accumulator: f64 = 0.0;
    let mut frame_time: Duration = Duration::zero();

    let mut ticks: u128 = 0;
    let mut is_initialized = false;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            WinitEvent::WindowEvent { event, window_id } => match event {
                WinitWindowEvent::CloseRequested => {
                    if window_id == window.id() {
                        *control_flow = ControlFlow::Exit
                    }
                }
                WinitWindowEvent::Resized(size) => {
                    println!("[Window] Resized to ({}, {})", size.width, size.height);

                    renderer.resize(size.width, size.height);
                    window.request_redraw();
                }
                _ => {}
            },
            WinitEvent::MainEventsCleared => {
                let new_time = Instant::now();
                frame_time = new_time - current_time;
                current_time = new_time;

                if !is_initialized {
                    init_callback(&mut renderer, &mut state);
                    is_initialized = true;

                    renderer.rebuild_swapchain();
                }

                // To avoid timing inconsistencies and errors, snap the delta time to the target delta time if it is within some small threshold.
                let snapped_delta_time_ms = {
                    let millis = frame_time.whole_microseconds() as f64 / 1000.0;
                    if (millis.abs() - target_dt) < 0.0002 {
                        target_dt
                    } else {
                        millis
                    }
                };

                let snapped_delta_time_seconds = snapped_delta_time_ms / 1000.0;
                accumulator += snapped_delta_time_seconds;
                while accumulator >= target_dt {
                    tick_callback(&mut state);

                    accumulator -= target_dt;
                    time += target_dt;
                    ticks += 1;

                    fps_counter += 1;
                }

                fps_timer = fps_timer + frame_time;
                if fps_timer >= one_second {
                    fps_timer = time::Duration::zero();
                    fps = fps_counter;
                    fps_counter = 0;

                    println!("FPS: {}", fps);
                }

                window.request_redraw();
            }
            WinitEvent::RedrawRequested(_) => {
                render_callback(ticks, &mut renderer, &state);
            }
            _ => (),
        }
    });
}
