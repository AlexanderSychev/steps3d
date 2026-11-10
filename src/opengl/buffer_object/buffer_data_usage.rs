use gl::{
    DYNAMIC_COPY, DYNAMIC_DRAW, DYNAMIC_READ,
    STATIC_COPY, STATIC_DRAW, STATIC_READ,
    STREAM_COPY, STREAM_DRAW, STREAM_READ
};

/// Стратегия использования буфера данных
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum BufferDataUsageStrategy {
    /// Данные задаются один раз и будут использованы **немного** раз
    Stream,
    /// Данные задаются один раз и будут использованы **много** раз
    Static,
    /// Данные задаются часто и будут использованы **много** раз
    Dynamic,
}

/// Назначение буфера данных
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum BufferDataUsageType {
    /// Данные будут использоваться рендерингом
    Draw,
    /// Данные будут читаться приложением
    Read,
    /// Данные будут использоваться и рендерингом, и приложением
    Copy,
}

/// Правила использования буфера данных
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct BufferDataUsage {
    // Стратегия использования буфера данных
    pub strategy: BufferDataUsageStrategy,
    // Назначение буфера данных
    pub usage_type: BufferDataUsageType,
}

impl BufferDataUsage {
    /// Создать новые правила использования буфера данных
    pub fn new(strategy: BufferDataUsageStrategy, usage_type: BufferDataUsageType) -> Self {
        Self {
            strategy,
            usage_type,
        }
    }
}

impl Into<u32> for BufferDataUsage {
    fn into(self) -> u32 {
        match (self.strategy, self.usage_type) {
            (BufferDataUsageStrategy::Stream, BufferDataUsageType::Draw) => STREAM_DRAW,
            (BufferDataUsageStrategy::Stream, BufferDataUsageType::Read) => STREAM_READ,
            (BufferDataUsageStrategy::Stream, BufferDataUsageType::Copy) => STREAM_COPY,
            (BufferDataUsageStrategy::Static, BufferDataUsageType::Draw) => STATIC_DRAW,
            (BufferDataUsageStrategy::Static, BufferDataUsageType::Read) => STATIC_READ,
            (BufferDataUsageStrategy::Static, BufferDataUsageType::Copy) => STATIC_COPY,
            (BufferDataUsageStrategy::Dynamic, BufferDataUsageType::Draw) => DYNAMIC_DRAW,
            (BufferDataUsageStrategy::Dynamic, BufferDataUsageType::Read) => DYNAMIC_READ,
            (BufferDataUsageStrategy::Dynamic, BufferDataUsageType::Copy) => DYNAMIC_COPY,
        }
    }
}
