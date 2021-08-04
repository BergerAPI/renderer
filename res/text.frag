#version 330 core
in vec2 TexCoords;
flat in vec4 fg;
flat in vec4 bg;
uniform int backgroundPass;

layout(location = 0, index = 0) out vec4 color;
layout(location = 0, index = 1) out vec4 alphaMask;

uniform sampler2D mask;

#define COLORED 2

void main() {
    if ((int(fg.a) & COLORED) != 0) {
        vec4 glyphColor = texture(mask, TexCoords);
        alphaMask = vec4(glyphColor.a);

        if (glyphColor.a != 0) {
            glyphColor.rgb = vec3(glyphColor.rgb / glyphColor.a);
        }

        color = vec4(glyphColor.rgb, 1.0);
    } else {
        vec3 textColor = texture(mask, TexCoords).rgb;
        alphaMask = vec4(textColor, textColor.r);
        color = vec4(fg.rgb, 1.0);
    }
}