# GLM vs glam

Некоторые соответствия между библиотеками `GLM` (C++) и `glam` (Rust).

## Векторы и свиззлинг (комбинация компонент)

GLM:
```c++
glm::vec3 a = glm::vec3(1.0f, 2.0f, 3.0f);
glm::vec3 b = a.xyz;    // или a.xyz() в зависимости от настроек
glm::vec3 c (a.xyz);
glm::vec3 d (a.xyz());
glm::vec3 e (a.x, a.yz); // конкатенация скаляра и vec2
glm::vec3 g (1);         // вектор (1, 1, 1)
glm::vec3 h = a + b;
```

glam:
```rust
let a = Vec3::new(1.0, 2.0, 3.0);
let b = a;               // Копирование (Vec3 реализует Copy)
let c = a.xyz();         // Честный свиззлинг через встроенные методы
let d = a.xyz();
let e = Vec3::new(a.x, a.y, a.z); // В glam нет прямой склейки f32 + Vec2 конструктором, передаем явно
let g = Vec3::ONE;       // Или Vec3::splat(1.0) для заполнения всех компонент единицей
let h = a + b;
```

## Произведения и 4D-векторы

GLM:
```c++
float r = glm::dot(a, b);   // скалярное
glm::vec3 p = glm::cross(b, c); // векторное
glm::vec4 a4 = glm::vec4(1.0f, 2.0f, 3.0f, 4.0f);
glm::vec4 b4 (a4.xy, a4.z, a4.w);
```

glam:
```rust
let r = a.dot(b);           // Метод прямо на векторе
let p = b.cross(c);
let a4 = Vec4::new(1.0, 2.0, 3.0, 4.0);
let b4 = Vec4::new(a4.x, a4.y, a4.z, a4.w); // Либо a4.xyz().extend(a4.w)
```

## Матрицы (Линейная алгебра)

GLM:
```c++
glm::mat3 m1(1.0f);            // Единичная матрица
glm::vec3 a1 = m1 * a;         // Умножение матрицы на вектор
glm::mat3 mm(...);             // Заполнение по строкам/столбцам
glm::mat3 mmi = glm::inverse(mm); // Обратная матрица
glm::mat3 n(0, 1, 2, 3, 4, 5, 6, 7, 8); // Инициализация массивом
glm::mat3 t = transpose(n);
```

glam:
```rust
let m1 = Mat3::IDENTITY;       // Константа единичной матрицы
let a1 = m1 * a;               // Умножение реализовано через прегрузку оператора *
let mm = Mat3::from_cols(      // В glam заполнение идет строго по столбцам (column-major)
    Vec3::new(0.6, 0.2, 0.3),
    Vec3::new(0.2, 0.7, 0.5),
    Vec3::new(0.3, 0.5, 0.7),
);
let mmi = mm.inverse();        // Метод матрицы
let n = Mat3::from_cols_array(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
let t = n.transpose();
```

## Кватернионы и углы Эйлера (Ориентация в 3D)

GLM:
```c++
glm::quat q1 = glm::angleAxis(M_PI * 0.25f, glm::vec3(0, 0, 1)); // Поворот вокруг оси Z
glm::quat n_q = glm::normalize(q1);
float l1 = glm::length(n_q);   // Длина кватерниона
glm::quat q2(1.0f, 0.0f, 0.0f, 1.0f); // (w, x, y, z) в GLM!

// Углы Эйлера
float roll  = glm::roll(q1);
float pitch = glm::pitch(q1);
float yaw   = glm::yaw(q1);
glm::vec3 angles = glm::eulerAngles(q1);

// Сферическая интерполяция (SLERP) и перевод в матрицу
glm::quat q3 = glm::slerp(q1, q2, 0.2f);
glm::mat4 rotate = glm::mat4_cast(q1);
```

glam:
```rust
// Конструктор принимает угол в радианах и нормализованную ось поворота
let q1 = Quat::from_axis_angle(Vec3::Z, std::f32::consts::FRAC_PI_4);
let n_q = q1.normalize();
let l1 = n_q.length();
// ОЧЕНЬ ВАЖНО: В glam кватернион принимает параметры в порядке (x, y, z, w)
let q2 = Quat::from_xyzw(0.0, 0.0, 1.0, 1.0);

// Получение углов Эйлера в glam делается одним методом во избежание Gimbal Lock
// Возвращает кортеж (roll, pitch, yaw) в зависимости от выбранного порядка осей (например, YXZ)
let (roll, pitch, yaw) = q1.to_euler(glam::EulerRot::YXZ);
let angles = Vec3::new(roll, pitch, yaw);

// Интерполяция и каст в матрицу 4x4
let q3 = q1.slerp(q2, 0.2);
let rotate = Mat4::from_quat(q1); // Прямой перевод кватерниона в матрицу трансформации 4x4
```
