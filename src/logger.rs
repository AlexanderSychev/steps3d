//! Модуль логирования

use simple_error::{SimpleResult, simple_error};

/// Инициализация логгера
pub fn init_logger() -> SimpleResult<()> {
    use simplelog::{ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode};
    let config = ConfigBuilder::new().set_time_format_rfc3339().build();
    TermLogger::init(
        LevelFilter::Trace,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .map_err(|err| simple_error!(err.to_string()))
}
