#version 430 core

layout(location = 1) in vec3 position;
layout(location = 2) in vec4 aColor;
layout(location = 3) in vec3 aNorms;

out vec4 color;
uniform mat4 matrix;
uniform float time;
out vec3 norms;

void main()
{
    gl_Position = matrix * vec4(position, 1.0);
    color = aColor;
    norms = aNorms;
}