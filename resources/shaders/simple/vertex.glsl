// Простейший вершинный шейдер: преобразует позицию вершины в clip space

#version 330 core

layout (location = 0) in vec3 in_position;

uniform mat4 model_view_projection;

void main() {
  gl_Position = model_view_projection * vec4(in_position, 1.0);
}
