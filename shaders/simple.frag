#version 430 core

in vec4 color;
in vec3 norms;

out vec4 fragColor;

void main()
{
    //vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));
    vec3 lightDirection = normalize(vec3(0.1, -0.1, 0.1));
    vec3  norms_new = normalize(norms);
    float diffuse = max(0.0, dot(norms_new, -lightDirection));
    fragColor = vec4(color.rgb * diffuse, color.a);
}