use crate::gl;

use gl::types::*;
use std::fmt::{self, Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum ShaderError {
    Io(io::Error),
    Compile(String),
    Link(String),
}

impl std::error::Error for ShaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ShaderError::Io(err) => err.source(),
            _ => None,
        }
    }
}

impl Display for ShaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ShaderError::Io(err) => write!(f, "Unable to read shader: {}", err),
            ShaderError::Compile(log) => {
                write!(f, "Failed compiling shader: {}", log)
            }
            ShaderError::Link(log) => write!(f, "Failed linking shader: {}", log),
        }
    }
}

#[derive(Debug)]
pub struct Shader {
    pub id: GLuint,
}

#[derive(Debug)]
pub struct Program {
    pub id: GLuint,
}

impl Shader {
    pub fn new(kind: GLenum, source: &'static str) -> Result<Shader, ShaderError> {
        unsafe {
            let len: [GLint; 1] = [source.len() as GLint];
            let shader = {
                let shader = gl::CreateShader(kind);
                gl::ShaderSource(shader, 1, &(source.as_ptr() as *const _), len.as_ptr());
                gl::CompileShader(shader);
                shader
            };

            let mut success: GLint = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

            if success == GLint::from(gl::TRUE) {
                Ok(Self { id: shader })
            } else {
                let log = get_shader_info_log(shader);

                gl::DeleteShader(shader);

                Err(ShaderError::Compile(log))
            }
        }
    }

    pub fn attach(&self, program: GLuint) {
        unsafe {
            gl::AttachShader(program, self.id);
        }
    }
}

impl Program {
    pub fn new(vertex: Shader, fragment: Shader) -> Result<Program, ShaderError> {
        unsafe {
            let program = gl::CreateProgram();
            let mut success: GLint = 0;

            vertex.attach(program);
            fragment.attach(program);

            gl::LinkProgram(program);
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);

            if success == i32::from(gl::TRUE) {
                Ok(Self { id: program })
            } else {
                Err(ShaderError::Link(get_program_info_log(program)))
            }
        }
    }
}

fn get_program_info_log(program: GLuint) -> String {
    unsafe {
        let mut max_length: GLint = 0;
        let mut actual_length: GLint = 0;
        let mut buf: Vec<u8> = Vec::with_capacity(max_length as usize);

        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut max_length);

        gl::GetProgramInfoLog(
            program,
            max_length,
            &mut actual_length,
            buf.as_mut_ptr() as *mut _,
        );

        buf.set_len(actual_length as usize);
        String::from_utf8(buf).unwrap()
    }
}

fn get_shader_info_log(shader: GLuint) -> String {
    unsafe {
        let mut max_length: GLint = 0;
        let mut actual_length: GLint = 0;
        let mut buf: Vec<u8> = Vec::with_capacity(max_length as usize);

        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut max_length);
        gl::GetShaderInfoLog(
            shader,
            max_length,
            &mut actual_length,
            buf.as_mut_ptr() as *mut _,
        );

        buf.set_len(actual_length as usize);
        String::from_utf8(buf).unwrap()
    }
}
