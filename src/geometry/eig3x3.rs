use glam::{Mat3, Vec3};

use super::constants::DEFAULT_GEOMETRY_EPSILON;

/// Структура для возврата результатов спектрального разложения матрицы
#[derive(Debug, Clone)]
pub struct EigenSystem3x3 {
    /// Собственные значения (числа), отсортированные по возрастанию или убыванию
    pub values: Vec3,
    /// Столбцы этой матрицы являются нормированными собственными векторами
    pub vectors: Mat3,
}

/// Нахождение собственных чисел и векторов симметричной матрицы 3x3 (Аналог библиотеки eig3x3)
/// Матрица ОБЯЗАТЕЛЬНО должна быть симметричной (M == M.transpose())
pub fn eigen_symmetric_3x3(m: &Mat3, maybe_epsilon: Option<f32>) -> Option<EigenSystem3x3> {
    let epsilon = maybe_epsilon.unwrap_or(DEFAULT_GEOMETRY_EPSILON);

    // Извлекаем компоненты симметричной матрицы
    // m.x_axis = [a, b, c], m.y_axis = [b, d, e], m.z_axis = [c, e, f]
    let a = m.x_axis.x;
    let b = m.x_axis.y; // Он же m.y_axis.x
    let c = m.x_axis.z; // Он же m.z_axis.x
    let d = m.y_axis.y;
    let e = m.y_axis.z; // Он же m.z_axis.y
    let f = m.z_axis.z;

    // Шаг 1: Вычисляем инварианты характеристического уравнения (полинома)
    let trace = a + d + f; // След матрицы
    let p1 = b * b + c * c + e * e;

    // Если внедиагональные элементы близки к нулю, матрица уже диагональная
    if p1 < epsilon {
        return Some(EigenSystem3x3 {
            values: Vec3::new(a, d, f),
            vectors: Mat3::IDENTITY,
        });
    }

    let q = (trace - (a * d + a * f + d * f - p1)) / 3.0; // Вспомогательный инвариант
    let p2 = (a - q) * (d - q) * (f - q) + 2.0 * b * c * e
        - (a - q) * e * e
        - (d - q) * c * c
        - (f - q) * b * b;
    let p = (p1 + (a - q).powi(2) + (d - q).powi(2) + (f - q).powi(2)) / 6.0;

    let r = p2 / 2.0;

    // Защита от ошибок округления f32
    let phi = if p > 0.0 {
        let cos_phi = (r / p.powf(1.5)).clamp(-1.0, 1.0);
        cos_phi.acos() / 3.0
    } else {
        0.0
    };

    // Вычисляем три собственных числа по тригонометрической формуле Виета-Кардано
    let sqrt_p = p.sqrt();
    let mut lam1 = q + 2.0 * sqrt_p * phi.cos();
    let mut lam2 = q + 2.0 * sqrt_p * (phi + 2.0 * std::f32::consts::FRAC_PI_3).cos();
    let mut lam3 = q + 2.0 * sqrt_p * (phi + 4.0 * std::f32::consts::FRAC_PI_3).cos();

    // Сортируем собственные значения по возрастанию для стабильности
    if lam1 > lam2 {
        std::mem::swap(&mut lam1, &mut lam2);
    }
    if lam1 > lam3 {
        std::mem::swap(&mut lam1, &mut lam3);
    }
    if lam2 > lam3 {
        std::mem::swap(&mut lam2, &mut lam3);
    }
    let values = Vec3::new(lam1, lam2, lam3);

    // Шаг 2: Нахождение собственных векторов методом вырожденных матриц (обратная итерация)
    let mut vectors = Mat3::ZERO;
    let l_arr = [lam1, lam2, lam3];

    for i in 0..3 {
        // Строим характеристическую матрицу (M - lambda * I)
        let mut shifted = *m;
        shifted.x_axis.x -= l_arr[i];
        shifted.y_axis.y -= l_arr[i];
        shifted.z_axis.z -= l_arr[i];

        // Ищем вектор через векторное произведение строк (ядро линейного оператора)
        let r0 = shifted.x_axis;
        let r1 = shifted.y_axis;
        let r2 = shifted.z_axis;

        // Берем пары строк и ищем ненулевое векторное произведение
        let mut v = r0.cross(r1);
        if v.length_squared() < epsilon {
            v = r0.cross(r2);
        }
        if v.length_squared() < epsilon {
            v = r1.cross(r2);
        }

        // Если все векторные произведения занулились, у нас кратные корни (сфера / одинаковые оси)
        if v.length_squared() < epsilon {
            // Задаем ортогональный дефолтный базис
            v = match i {
                0 => Vec3::X,
                1 => Vec3::Y,
                _ => Vec3::Z,
            };
        }

        let v_norm = v.normalize();

        // Записываем собственный вектор в соответствующий столбец результирующей матрицы
        match i {
            0 => vectors.x_axis = v_norm,
            1 => vectors.y_axis = v_norm,
            _ => vectors.z_axis = v_norm,
        }
    }

    Some(EigenSystem3x3 { values, vectors })
}
