use simple_error::SimpleResult;

use crate::opengl::CapabilityManager;

mod app;
mod canvas;
mod color;
mod geometry;
mod graphics;
mod light;
mod logger;
mod opengl;
mod ray_tracing;
mod sdl_window;

fn main() -> SimpleResult<()> {
    use sdl2::event::{Event, WindowEvent};
    use sdl2::keyboard::Keycode;
    use std::time::{Duration, Instant};

    logger::init_logger()?;

    const WIDTH: u32 = 1024;
    const HEIGHT: u32 = 768;

    // Начальная инициализация
    let window = sdl_window::SdlGlWindow::try_new("Steps 3D - OpenGL", WIDTH, HEIGHT)?;
    opengl::load_with(|name| window.gl_get_proc_address(name));

    // "Опрос" устройства
    let mut capabilities = CapabilityManager::new(false);

    let _debug_logger = if opengl::log::supports_debug_output() {
        Some(opengl::log::attach_log(
            |source, message_type, id, severity, message| {
                use log::{debug, info, trace, warn};
                let text = format!(
                    "[OPENGL][#{}][SOURCE: {:?}][TYPE: {:?}]: {}",
                    id, source, message_type, message
                );
                match severity {
                    opengl::log::DebugLogSeverity::DebugSeverityHigh => warn!("{}", text),
                    opengl::log::DebugLogSeverity::DebugSeverityMedium => info!("{}", text),
                    opengl::log::DebugLogSeverity::DebugSeverityLow => debug!("{}", text),
                    opengl::log::DebugLogSeverity::DebugSeverityNotification => trace!("{}", text),
                }
            },
        )?)
    } else {
        log::warn!("OpenGL debug output is not supported by this context");
        None
    };

    // Отдельно включить отсечение и задать параметры лицевых граней.
    capabilities.enable(opengl::Capability::CullFace);
    opengl::configuration::FrontFaceDirection::Clockwise.set();
    opengl::configuration::CullFaceMode::Back.set();

    // Инициализировать настройки буферов кадра.
    let color_buffer = opengl::Buffer::default_color();
    let depth_buffer = opengl::Buffer::default_depth();
    let stencil_buffer = opengl::Buffer::default_stencil();
    opengl::Buffer::clear_buffers(&[color_buffer, depth_buffer, stencil_buffer]);

    opengl::configuration::set_viewport_size(WIDTH, HEIGHT)?;
    opengl::errors::check_errors()?;

    const TARGET_FRAME_TIME: Duration = Duration::from_nanos(16_666_667);
    let mut frame_delta = app::FrameDelta::default();
    let mut event_pump = window.event_pump()?;
    'running: loop {
        let frame_start = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode, .. } => {
                    if keycode == Some(Keycode::Escape) {
                        break 'running;
                    }
                }
                Event::Window {
                    win_event: WindowEvent::SizeChanged(width, height),
                    ..
                } => {
                    if let (Ok(width), Ok(height)) = (u32::try_from(width), u32::try_from(height)) {
                        if width > 0 && height > 0 {
                            opengl::configuration::set_viewport_size(width, height)?;
                            opengl::errors::check_errors()?;
                        }
                    }
                }
                _ => {}
            }
        }

        let remaining = TARGET_FRAME_TIME.saturating_sub(frame_start.elapsed());
        if !remaining.is_zero() {
            std::thread::sleep(remaining);
        }

        frame_delta.tick();
    }

    Ok(())
}
