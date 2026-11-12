//! Провоцирующая вершина

use gl::{
    FIRST_VERTEX_CONVENTION, GetIntegerv, LAST_VERTEX_CONVENTION, PROVOKING_VERTEX, ProvokingVertex,
};
use simple_error::{SimpleError, SimpleResult, simple_error};
use strum::FromRepr;

/// Тип используемой провоцирующей вершины
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr, Default)]
#[repr(u32)]
pub enum ProvokingVertexConvention {
    /// Первая вершина
    FirstVertex = FIRST_VERTEX_CONVENTION,
    /// Последняя вершина (значение по-умолчанию согласно конвенциям OpenGL)
    #[default]
    LastVertex = LAST_VERTEX_CONVENTION,
}

impl PartialEq<u32> for ProvokingVertexConvention {
    fn eq(&self, other: &u32) -> bool {
        *self as u32 == *other
    }
}

impl TryFrom<u32> for ProvokingVertexConvention {
    type Error = SimpleError;

    fn try_from(value: u32) -> SimpleResult<Self> {
        Self::from_repr(value).ok_or_else(|| {
            simple_error!(
                "Cannot create \"ProvokingVertexConvention\" value from \"u32\" value: {}",
                value
            )
        })
    }
}

/// Менеджер состояния провоцирующей вершины с кэшированием
#[derive(Debug, Default)]
pub struct ProvokingVertexState {
    /// Закэшированный тип провоцирующей вершины
    convention: ProvokingVertexConvention,
}

impl ProvokingVertexState {
    /// Попробовать создать менеджер состояния провоцирующей вершины
    pub fn try_new() -> SimpleResult<Self> {
        let mut result = Self::default();
        result.refresh()?;
        Ok(result)
    }

    /// Получить текущую провоцирующую вершину
    pub fn get_convention(&self) -> ProvokingVertexConvention {
        self.convention
    }

    /// Установить провоцирующую вершину
    pub fn set_convention(&mut self, convention: ProvokingVertexConvention) {
        // Если вершина не поменялась, то прерываем операцию, чтобы не обращаться к GPU
        if self.convention == convention {
            return;
        }

        // Сначала обращаемся к GPU, а потом обновляем кэш
        unsafe {
            ProvokingVertex(convention as u32);
        }
        self.convention = convention;
    }

    /// Обновить закэшированное состояние из GPU
    pub fn refresh(&mut self) -> SimpleResult<()> {
        // Запрашиваем значение у GPU
        let mut open_gl_value = 0;
        unsafe {
            GetIntegerv(PROVOKING_VERTEX, &mut open_gl_value);
        }

        // Безопасное преобразование в u32
        let open_gl_value =
            u32::try_from(open_gl_value).map_err(|err| simple_error!(err.to_string()))?;

        // Безопасное преобразование в ProvokingVertexConvention
        let convention = ProvokingVertexConvention::try_from(open_gl_value)?;

        // Кэшируем значение
        self.convention = convention;

        Ok(())
    }
}
