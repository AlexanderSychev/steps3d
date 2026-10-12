use std::ops::{Add, Mul, Neg};

/// Двухмерный вектор
#[derive(Debug, Clone, Copy)]
pub struct Vector2D {
    /// Координата X
    pub x: f32,
    /// Координата Y
    pub y: f32,
}

impl Vector2D {
    /// Скалярное произведение векторов
    pub fn dot_product(u: &Vector2D, v: &Vector2D) -> f32 {
        u.x * v.x + u.y * v.y
    }

    /// Косинус угла между двумя векторами
    pub fn angle_cos(u: &Vector2D, v: &Vector2D) -> Option<f32> {
        let dot = Vector2D::dot_product(u, v);
        let len_u = u.len();
        let len_v = v.len();
        let len_product = len_u * len_v;
        if len_product > 0.0 {
            // "Подрезаем" косинус, чтобы он был в диапазоне [-1.0, 1.0]
            Some((dot / len_product).clamp(-1.0, 1.0))
        } else {
            None
        }
    }

    /// Угол между двумя векторами в радианах
    pub fn angle_rad(u: &Vector2D, v: &Vector2D) -> Option<f32> {
        let maybe_cos = Vector2D::angle_cos(u, v);
        maybe_cos.map(|cos| cos.acos())
    }

    /// Длина (норма) вектора
    pub fn len(&self) -> f32 {
        Vector2D::dot_product(self, self).sqrt()
    }

    /// Получить нормализованную версию вектора
    pub fn normalized(&self) -> Option<Vector2D> {
        let len = self.len();
        if len > 0.0 {
            let inv_len = 1.0 / len; // CPU быстрее умножает, чем делит
            Some(Vector2D {
                x: self.x * inv_len,
                y: self.y * inv_len,
            })
        } else {
            None
        }
    }
}

// Сложение векторов
impl Add<Vector2D> for Vector2D {
    type Output = Vector2D;

    fn add(self, rhs: Vector2D) -> Vector2D {
        Vector2D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

// Обратный вектор
impl Neg for Vector2D {
    type Output = Vector2D;

    fn neg(self) -> Self::Output {
        Vector2D {
            x: -self.x,
            y: -self.y,
        }
    }
}

// Умножение вектора на число
impl Mul<f32> for Vector2D {
    type Output = Vector2D;

    fn mul(self, rhs: f32) -> Vector2D {
        Vector2D {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
impl Mul<Vector2D> for f32 {
    type Output = Vector2D;

    fn mul(self, rhs: Vector2D) -> Self::Output {
        rhs * self
    }
}

impl PartialEq<Vector2D> for Vector2D {
    fn eq(&self, other: &Vector2D) -> bool {
        // Тип float сам по себе не очень точен,
        // и может накапливать ошибки при сравнении дробей.
        // Нужно сравнивать с некоторой погрешностью.
        let eps = 1e-6;
        (self.x - other.x).abs() < eps && (self.y - other.y).abs() < eps
    }
}

#[cfg(test)]
mod vector2d_test {
    use super::*;

    /// Перегрузка оператора "+": тест сложения векторов
    #[test]
    fn test_add() {
        let v1 = Vector2D { x: 1.0, y: 2.0 };
        let v2 = Vector2D { x: 3.0, y: 4.0 };
        let result = v1 + v2;
        assert_eq!(result.x, 4.0);
        assert_eq!(result.y, 6.0);
    }

    /// Перегрузка унарного минуса: обратный вектор
    #[test]
    fn test_neg() {
        let v1 = Vector2D { x: 1.0, y: 2.0 };
        let v2 = Vector2D { x: -1.0, y: -2.0 };

        assert_eq!(-v1, v2);
    }

    /// Перегрузка оператора "*": тест перемножения на число
    #[test]
    fn test_mul() {
        let v1 = Vector2D { x: 1.0, y: 2.0 };

        let vector_on_fload = v1 * 2.0;
        assert_eq!(vector_on_fload.x, 2.0);
        assert_eq!(vector_on_fload.y, 4.0);

        let float_on_vector = 2.0 * v1;
        assert_eq!(float_on_vector.x, 2.0);
        assert_eq!(float_on_vector.y, 4.0);
    }

    /// Перегрузка операторов "==" и "!=": тест сравнения
    #[test]
    fn test_partial_eq() {
        let v1 = Vector2D { x: 1.0, y: 2.0 };
        let v2 = Vector2D { x: 1.0, y: 2.0 };
        assert_eq!(v1, v2);
    }

    /// Проверка базовых математических утверждений
    #[test]
    fn test_vector_laws() {
        let u = Vector2D { x: 1.0, y: 2.0 };
        let v = Vector2D { x: 3.0, y: 4.0 };
        let w = Vector2D { x: 2.0, y: 5.0 };
        let a = 2.0f32;
        let b = 3.0f32;
        let zero = Vector2D { x: 0.0, y: 0.0 };

        // u + v = v + u
        assert_eq!(u + v, v + u);

        // (u + v) + w = u + (v + w)
        assert_eq!((u + v) + w, u + (v + w));

        // u + O = u
        assert_eq!(u + zero, u);

        // 1 * u = u
        assert_eq!(1.0 * u, u);

        // a * (u + v) = a * u + a * v
        assert_eq!(a * (u + v), a * u + a * v);

        // (a + b) * u = a * u + b * u
        assert_eq!((a + b) * u, a * u + b * u);

        // u + (-u) = O
        assert_eq!(u + (-u), zero);

        // -u = -1 * u
        assert_eq!(-u, -1.0 * u);
    }

    /// Проверка математических утверждений для скалярного произведения
    #[test]
    fn test_dot_product() {
        let u = Vector2D { x: 1.0, y: 2.0 };
        let v = Vector2D { x: 3.0, y: 4.0 };
        let w = Vector2D { x: 2.0, y: 5.0 };
        let a = 2.0f32;

        // (u, v) = (v, u)
        assert_eq!(Vector2D::dot_product(&u, &v), Vector2D::dot_product(&v, &u));

        // (u + v, w) = (u, w) + (v, w)
        assert_eq!(
            Vector2D::dot_product(&(u + v), &w),
            Vector2D::dot_product(&u, &w) + Vector2D::dot_product(&v, &w)
        );

        // (a * u, v) = a * (u, w)
        assert_eq!(
            Vector2D::dot_product(&(a * u), &v),
            a * Vector2D::dot_product(&u, &v)
        );

        // (u, u) >= 0
        assert!(Vector2D::dot_product(&u, &u) >= 0.0);
    }

    /// Проверка математических утверждений для длины вектора
    #[test]
    fn test_len() {
        let u = Vector2D { x: 1.0, y: 2.0 };
        let v = Vector2D { x: 3.0, y: 4.0 };
        let zero = Vector2D { x: 0.0, y: 0.0 };
        let a = 2.0f32;

        // ||a * u|| = |a| * ||u||
        assert_eq!((a * u).len(), a.abs() * u.len());

        // ||u + v|| <= ||u|| + ||v||
        assert!((u + v).len() <= u.len() + v.len());

        // ||u|| = 0 when u = O
        assert_eq!(zero.len(), 0.0);

        // |(u, v)| <= ||u|| * ||v||
        assert!(Vector2D::dot_product(&u, &v).abs() <= u.len() * v.len());
    }
}

/// Матрица 2 на 2
#[derive(Debug, Clone, Copy)]
struct Matrix2x2((f32, f32), (f32, f32));

impl Matrix2x2 {
    /// Создать и вернуть нулевую матрицу
    pub fn zero() -> Matrix2x2 {
        Matrix2x2((0.0, 0.0), (0.0, 0.0))
    }

    /// Создать и вернуть единичную матрицу
    pub fn identity() -> Matrix2x2 {
        Matrix2x2((1.0, 0.0), (0.0, 1.0))
    }

    /// Создать и вернуть матрицу масштабирования
    pub fn scale(l: f32, m: f32) -> Matrix2x2 {
        Matrix2x2((l, 0.0), (0.0, m))
    }

    /// Создать и вернуть матрицу отражения по оси X
    pub fn mirror_x() -> Matrix2x2 {
        Matrix2x2((1.0, 0.0), (0.0, -1.0))
    }

    /// Создать и вернуть матрицу отражения по оси Y
    pub fn mirror_y() -> Matrix2x2 {
        Matrix2x2((-1.0, 0.0), (0.0, 1.0))
    }

    /// Создать и вернуть матрицу поворота
    pub fn rotation(angle: f32) -> Matrix2x2 {
        Matrix2x2((angle.cos(), angle.sin()), (-angle.sin(), angle.cos()))
    }

    /// Создать и вернуть матрицу сдвига
    pub fn shear(x: f32, y: f32) -> Matrix2x2 {
        Matrix2x2((1.0, x), (y, 1.0))
    }

    /// Транспонировать матрицу
    pub fn transpose(&self) -> Matrix2x2 {
        Matrix2x2((self.0.0, self.1.0), (self.0.1, self.1.1))
    }

    /// Детерминант матрицы
    pub fn det(&self) -> f32 {
        self.0.0 * self.1.1 - self.0.1 * self.1.0
    }

    /// Получить обратную матрицу
    pub fn invert(&self) -> Option<Matrix2x2> {
        let det = self.det();
        if det != 0.0 {
            let inverse_det = 1.0 / det;
            Some(inverse_det * Matrix2x2((self.1.1, -self.0.1), (-self.1.0, self.0.0)))
        } else {
            None
        }
    }
}

// Перегрузка оператора "+"
impl Add<Matrix2x2> for Matrix2x2 {
    type Output = Matrix2x2;

    fn add(self, rhs: Matrix2x2) -> Self::Output {
        Matrix2x2(
            (self.0.0 + rhs.0.0, self.0.1 + rhs.0.1),
            (self.1.0 + rhs.1.0, self.1.1 + rhs.1.1),
        )
    }
}

// Перегрузка оператора "*"
impl Mul<f32> for Matrix2x2 {
    type Output = Matrix2x2;

    fn mul(self, rhs: f32) -> Self::Output {
        Matrix2x2(
            (self.0.0 * rhs, self.0.1 * rhs),
            (self.1.0 * rhs, self.1.1 * rhs),
        )
    }
}
impl Mul<Matrix2x2> for f32 {
    type Output = Matrix2x2;

    fn mul(self, rhs: Matrix2x2) -> Self::Output {
        rhs * self
    }
}
impl Mul<Matrix2x2> for Matrix2x2 {
    type Output = Matrix2x2;

    fn mul(self, rhs: Matrix2x2) -> Self::Output {
        Matrix2x2(
            (
                self.0.0 * rhs.0.0 + self.0.1 * rhs.1.0,
                self.0.0 * rhs.0.1 + self.0.1 * rhs.1.1,
            ),
            (
                self.1.0 * rhs.0.0 + self.1.1 * rhs.1.0,
                self.1.0 * rhs.0.1 + self.1.1 * rhs.1.1,
            ),
        )
    }
}

impl Mul<Vector2D> for Matrix2x2 {
    type Output = Vector2D;

    fn mul(self, rhs: Vector2D) -> Self::Output {
        Vector2D {
            x: self.0.0 * rhs.x + self.0.1 * rhs.y,
            y: self.1.0 * rhs.x + self.1.1 * rhs.y,
        }
    }
}

impl PartialEq<Matrix2x2> for Matrix2x2 {
    fn eq(&self, other: &Matrix2x2) -> bool {
        let eps = 1e-6;
        (self.0.0 - other.0.0).abs() < eps
            && (self.0.1 - other.0.1).abs() < eps
            && (self.1.0 - other.1.0).abs() < eps
            && (self.1.1 - other.1.1).abs() < eps
    }
}

#[cfg(test)]
mod matrix2x2_test {
    use std::f32::consts::PI;

    use super::*;

    #[test]
    fn test_rules() {
        let zero = Matrix2x2::zero();
        let identity = Matrix2x2::identity();
        let a = Matrix2x2((1.0, 2.0), (3.0, 4.0));
        let b = Matrix2x2((5.0, 6.0), (7.0, 8.0));
        let c = Matrix2x2((9.0, 10.0), (11.0, 12.0));
        let alpha = 2.5f32;

        // (A * B) * C C = A * (B * C)
        assert_eq!((a * b) * c, a * (b * c));

        // A * (B + C) = A * B + A * C
        assert_eq!(a * (b + c), a * b + a * c);

        // (alpha * A) * B = alpha * (A * B)
        assert_eq!((alpha * a) * b, alpha * (a * b));

        // A * I = I * A = A
        assert_eq!(a * identity, identity * a);
        assert_eq!(a * identity, a);

        // A * O = O * A = O
        assert_eq!(a * zero, zero * a);
        assert_eq!(a * zero, zero);

        // transpose(A * B) = transose(B) * transpose(A)
        assert_eq!((a * b).transpose(), b.transpose() * a.transpose());

        // transpose(A + B) = transpose(A) + transpose(B)
        assert_eq!((a + b).transpose(), a.transpose() + b.transpose());

        // transpose(alpha * A) = alpha * transpose(A)
        assert_eq!((alpha * a).transpose(), alpha * a.transpose());

        // transpose(transpose(A)) = A
        assert_eq!(a.transpose().transpose(), a);
    }

    /// Проверка математических утверждений для обратных матриц
    #[test]
    fn test_invert() {
        let a = Matrix2x2((1.0, 2.0), (3.0, 4.0));
        let b = Matrix2x2((5.0, 6.0), (7.0, 8.0));

        // Убедимся, что матрицы не вырожденная
        assert_ne!(a.det(), 0.0);
        assert_ne!(b.det(), 0.0);

        // Приготовим обратные матрицы
        let a_inverted = a.invert().unwrap();
        let b_inverted = b.invert().unwrap();

        // (A^-1)^-1 = A
        assert_eq!(a_inverted.invert().unwrap(), a);

        // det(A^-1) = 1 / det(A)
        assert_eq!(a_inverted.det(), 1.0 / a.det());

        // (AB)^-1 = B^-1 * A^-1
        assert_eq!((a * b).invert().unwrap(), b_inverted * a_inverted);

        // (transpose(A))^-1 = transpose(A^-1)
        assert_eq!(a.transpose().invert().unwrap(), a_inverted.transpose());
    }

    // Проверка математических утверждений для матриц и векторов
    #[test]
    fn test_mul_vector2d() {
        let a = Matrix2x2((1.0, 2.0), (3.0, 4.0));
        let b = Matrix2x2((5.0, 6.0), (7.0, 8.0));
        let u = Vector2D { x: 1.0, y: 2.0 };
        let v = Vector2D { x: 3.0, y: 4.0 };
        let alpha = 2.5f32;

        // (A + B) * u = A * u + B * u
        assert_eq!((a + b) * u, a * u + b * u);

        // A * (B * u) = (A * B) * u
        assert_eq!(a * (b * u), (a * b) * u);

        // A * (u + v) = A * u + A * v
        assert_eq!(a * (u + v), a * u + a * v);

        // A * (alpha * u) = alpha * A * u
        assert_eq!(a * (alpha * u), alpha * a * u);

        // (A * u, v) = (u, transpose(a) * v)
        assert_eq!(
            Vector2D::dot_product(&(a * u), &v),
            Vector2D::dot_product(&u, &(a.transpose() * v))
        );
    }

    #[test]
    fn test_scale() {
        let s = Matrix2x2::scale(2.0, 4.0);
        let s_inverted = s.invert().unwrap();

        assert_eq!(s_inverted, Matrix2x2::scale(0.5, 0.25));
    }

    #[test]
    fn test_mirror() {
        let x = Matrix2x2::mirror_x();
        let y = Matrix2x2::mirror_y();

        assert_eq!(x.invert().unwrap(), x);
        assert_eq!(y.invert().unwrap(), y);
    }

    #[test]
    fn test_rotation() {
        let rotation = Matrix2x2::rotation(PI / 180.0); // 45 градусов в радианах

        // det R(angle) = 1
        assert_eq!(rotation.det(), 1.0);

        assert_eq!(rotation.invert().unwrap(), rotation.transpose());
        assert_eq!(rotation.invert().unwrap(), Matrix2x2::rotation(-PI / 180.0));
    }

    #[test]
    fn test_shear() {
        let x = Matrix2x2::shear(1.5, 0.0);
        let y = Matrix2x2::shear(0.0, 1.5);

        assert_eq!(x.det(), y.det());
        assert_eq!(x.det(), 1.0);

        let angle = 30.0f32.to_radians();
        let mu = -angle.sin();
        let lambda = (1.0 - angle.cos()) / angle.sin();
        let nu = lambda;

        // R(angle) = H(l, 0.0) * H(0.0, m) * H(n, 0.0)
        assert_eq!(
            Matrix2x2::rotation(angle),
            Matrix2x2::shear(lambda, 0.0) * Matrix2x2::shear(0.0, mu) * Matrix2x2::shear(nu, 0.0)
        );
    }
}
