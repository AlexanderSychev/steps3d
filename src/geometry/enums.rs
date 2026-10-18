/// Расположение точки относительно отрезка
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointArea {
    /// Слева
    Left,
    /// Справа
    Right,
    /// В начале отрезка
    Start,
    /// В конце отрезка
    End,
    /// Позади отрезка
    Behind,
    /// Впереди отрезка
    Beyond,
    /// На отрезке
    Between,
}

/// Расположение относительно прямой
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineArea {
    InPositive,
    InNegative,
    InBoth,
}
