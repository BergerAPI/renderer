#version 330 core

// Cell properties.
layout(location = 0) in vec2 coords;

// Glyph properties.
layout(location = 1) in vec4 glyph;

// uv mapping.
layout(location = 2) in vec4 uv;

// Text foreground rgb packed together with cell flags. textColor.a
// are the bitflags; consult RenderingGlyphFlags in renderer/mod.rs
// for the possible values.
layout(location = 3) in vec4 textColor;

// Background color.
layout(location = 4) in vec4 backgroundColor;

out vec2 TexCoords;
flat out vec4 fg;
flat out vec4 bg;

// Terminal properties
uniform vec2 cellDim;
uniform vec4 projection;

uniform int backgroundPass;

#define WIDE_CHAR 1

void main() {
    vec2 projectionOffset = projection.xy;
    vec2 projectionScale = projection.zw;

    vec2 position;
    position.x = (gl_VertexID == 0 || gl_VertexID == 1) ? 1. : 0.;
    position.y = (gl_VertexID == 0 || gl_VertexID == 3) ? 0. : 1.;

    vec2 glyphSize = glyph.zw;
    vec2 glyphOffset = glyph.xy;
    glyphOffset.y = cellDim.y - glyphOffset.y;

    vec2 finalPosition = coords + glyphSize * position + glyphOffset;
    gl_Position =
        vec4(projectionOffset + projectionScale * finalPosition, 0.0, 1.0);

    vec2 uvOffset = uv.xy;
    vec2 uvSize = uv.zw;
    TexCoords = uvOffset + position * uvSize;

    bg = vec4(backgroundColor.rgb / 255.0, backgroundColor.a);
    fg = vec4(textColor.rgb / 255.0, textColor.a);
}