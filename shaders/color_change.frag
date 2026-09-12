#version 430 core

out vec4 color;
uniform float time;

void main()
{
    float r = (sin(time * 0.5) + 1.0) / 2.0;
    color = vec4(r, 1.0f, 1.0f, 1.0f);
}