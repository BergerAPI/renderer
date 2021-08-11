#version 330 core

layout(location = 0) in vec2 coords;
layout(location = 1) in vec4 glyph;
layout(location = 2) in vec4 uv;
layout(location = 3) in vec4 textColor;

out vec2 o_TexCoords;
flat out vec4 o_Color;

uniform vec2 cellDim;
uniform mat4 projection;

void main() {
    vec2 position = vec2((gl_VertexID == 0 || gl_VertexID == 1) ? 1. : 0.,
                         (gl_VertexID == 0 || gl_VertexID == 3) ? 0. : 1.);
    vec2 glyphPosition = vec2(0, cellDim.y - glyph.y);

    gl_Position = projection * vec4(coords + glyphPosition + glyph.zw * position, 0.0, 1.0);

    o_TexCoords = uv.xy + position * uv.zw;
    o_Color = vec4(textColor.rgb / 255.0, textColor.a);
}