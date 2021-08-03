#version 330 core

layout(location = 0) in vec2 coords;
layout(location = 1) in vec4 glyph;
layout(location = 2) in vec4 uv;
layout(location = 3) in vec4 textColor;

out vec2 TexCoords;
flat out vec4 fg;

uniform vec4 projection;

#define WIDE_CHAR 1

void main() {
    vec2 projectionOffset = projection.xy;
    vec2 projectionScale = projection.zw;

    vec2 glyphSize = glyph.zw;
    vec2 glyphOffset = glyph.xy;

    vec2 position;
    position.x = (gl_VertexID == 0 || gl_VertexID == 1) ? 1. : 0.;
    position.y = (gl_VertexID == 0 || gl_VertexID == 3) ? 0. : 1.;

    vec2 finalPosition = coords + glyphSize * position + glyphOffset;
    gl_Position = vec4(projectionOffset + projectionScale * finalPosition, 0.0, 1.0);

    vec2 uvOffset = uv.xy;
    vec2 uvSize = uv.zw;
    TexCoords = uvOffset + position * uvSize;

    fg = vec4(textColor.rgb / 255.0, textColor.a);
}