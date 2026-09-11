#version 430 core

layout(location = 1) in vec3 position;
layout(location = 2) in vec4 aColor;

out vec4 color;

void main()
{
    gl_Position = vec4(position, 1.0);
    color = aColor;
}