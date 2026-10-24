use glam::{Mat3, Quat, Vec3};

/// Преобразует TBN-базис (касательное пространство) в кватернион
pub fn tangent_space_to_quat(tangent: Vec3, binormal: Vec3, normal: Vec3) -> Quat {
    // Вычисляем компоненты кватерниона по формулам из книги
    let x = normal.y - binormal.z;
    let y = tangent.z - normal.x;
    let z = binormal.x - tangent.y;
    let w = 1.0 + tangent.x + binormal.y + normal.z;

    // Создаем кватернион и обязательно нормализуем его, как в листинге GLM
    Quat::from_xyzw(x, y, z, w).normalize()
}

/// Восстанавливает векторы TBN-базиса из кватерниона.
pub fn quat_to_tangent_space(q: Quat) -> (Vec3, Vec3, Vec3) {
    // Строим матрицу вращения 3x3 напрямую из кватерниона
    let matrix = Mat3::from_quat(q);

    // Векторы TBN — это столбцы (или строки, в зависимости от конвенции) данной матрицы.
    // glam использует column-major порядок:
    let tangent = matrix.x_axis;
    let binormal = matrix.y_axis;
    let normal = matrix.z_axis;

    (tangent, binormal, normal)
}
