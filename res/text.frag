#version 330 core

in vec2 o_TexCoords;
flat in vec4 o_Color;

layout(location = 0, index = 0) out vec4 color;
layout(location = 0, index = 1) out vec4 alphaMask;

uniform sampler2D mask;

#define COLORED 2

void main() {
    if ((int(o_Color.a) & COLORED) != 0) {
        vec4 glyphColor = texture(mask, o_TexCoords);
        alphaMask = vec4(glyphColor.a);

        if (glyphColor.a != 0) {
            glyphColor.rgb = vec3(glyphColor.rgb / glyphColor.a);
        }

        color = vec4(glyphColor.rgb, 1.0);
    } else {
        vec3 textColor = texture(mask, o_TexCoords).rgb;
        alphaMask = vec4(textColor, textColor.r);
        color = vec4(o_Color.rgb, 1.0);
    }
}