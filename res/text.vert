#version 330 core

layout(location = 0) in vec2 coords;
layout(location = 1) in vec4 glyph;
layout(location = 2) in vec4 uv;
layout(location = 3) in vec4 textColor;

out vec2 TexCoords;
flat out vec4 fg;

uniform vec2 cellDim;
uniform mat4 projection;

void main() {
    vec2 position;
    position.x = (gl_VertexID == 0 || gl_VertexID == 1) ? 1. : 0.;
    position.y = (gl_VertexID == 0 || gl_VertexID == 3) ? 0. : 1.;

    vec2 glyphSize = glyph.zw;
    vec2 glyphOffset = glyph.xy;
    glyphOffset.y = cellDim.y - glyphOffset.y;

    vec2 finalPosition = coords + glyphOffset + glyphSize * position;
    gl_Position = projection * vec4(finalPosition, 0.0, 1.0);

    vec2 uvOffset = uv.xy;
    vec2 uvSize = uv.zw;
    TexCoords = uvOffset + position * uvSize;

    fg = vec4(textColor.rgb / 255.0, textColor.a);
}