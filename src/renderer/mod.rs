mod shader;

use crate::gl;
use crate::vectors::Vec2f;

use gl::types::*;
use std::mem;

#[derive(Debug, Clone)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone)]
pub struct RenderRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: Rgb,
}

static FRAGMENT: &str = include_str!("../../res/base.frag");
static VERTEX: &str = include_str!("../../res/base.vert");

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Vertex {
    x: f32,
    y: f32,
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug)]
pub struct Renderer {
    vao: GLuint,
    vbo: GLuint,

    program: shader::Program,

    vertices: Vec<Vertex>,
}

impl Renderer {
    pub fn new() -> Result<Self, shader::ShaderError> {
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        let program = shader::Program::new(
            shader::Shader::new(gl::VERTEX_SHADER, VERTEX)?,
            shader::Shader::new(gl::FRAGMENT_SHADER, FRAGMENT)?,
        )?;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            let mut attribute_offset = 0;

            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as i32,
                attribute_offset as *const _,
            );
            gl::EnableVertexAttribArray(0);
            attribute_offset += mem::size_of::<f32>() * 2;

            gl::VertexAttribPointer(
                1,
                3,
                gl::UNSIGNED_BYTE,
                gl::TRUE,
                mem::size_of::<Vertex>() as i32,
                attribute_offset as *const _,
            );
            gl::EnableVertexAttribArray(1);

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        Ok(Self {
            vao,
            vbo,
            program,
            vertices: Vec::new(),
        })
    }

    pub fn draw(&mut self, size: Vec2f) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * mem::size_of::<Vertex>()) as isize,
                self.vertices.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );

            gl::UseProgram(self.program.id);

            gl::DrawArrays(gl::TRIANGLES, 0, self.vertices.len() as i32);

            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }

        self.vertices.clear();
    }

    pub fn rectangle(&mut self, size: Vec2f, rect: &RenderRect) {
        let x = rect.x / (size.x / 2.) - 1.0;
        let y = -rect.y / (size.y / 2.) + 1.0;
        let quad = [
            Vertex {
                x,
                y,
                r: rect.color.r,
                g: rect.color.g,
                b: rect.color.b,
            },
            Vertex {
                x,
                y: y - rect.height / (size.y / 2.),
                r: rect.color.r,
                g: rect.color.g,
                b: rect.color.b,
            },
            Vertex {
                x: x + rect.width / (size.x / 2.),
                y,
                r: rect.color.r,
                g: rect.color.g,
                b: rect.color.b,
            },
            Vertex {
                x: x + rect.width / (size.x / 2.),
                y: y - rect.height / (size.y / 2.),
                r: rect.color.r,
                g: rect.color.g,
                b: rect.color.b,
            },
        ];

        self.vertices.push(quad[0]);
        self.vertices.push(quad[1]);
        self.vertices.push(quad[2]);
        self.vertices.push(quad[2]);
        self.vertices.push(quad[3]);
        self.vertices.push(quad[1]);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
