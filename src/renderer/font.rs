use crate::renderer::shader::{Program, Shader, ShaderError};

use crate::gl;
use gl::types::*;

use fnv::FnvHasher;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::mem::size_of;
use std::ptr;

use gl_matrix::common::*;
use gl_matrix::mat4;

use crate::vectors::Vec2f;

use crossfont::{
    BitmapBuffer, Error as RasterizerError, FontDesc, FontKey, GlyphKey, Metrics, Rasterize,
    RasterizedGlyph, Rasterizer, Size, Slant, Style, Weight,
};

use bitflags::bitflags;

const BATCH_MAX: usize = 0x1_0000;
const ATLAS_SIZE: i32 = 2048;

static FRAGMENT: &str = include_str!("../../res/text.frag");
static VERTEX: &str = include_str!("../../res/text.vert");

bitflags! {
    #[repr(C)]
    struct RenderingGlyphFlags: u8 {
        const WIDE_CHAR = 0b0000_0001;
        const COLORED   = 0b0000_0010;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Glyph {
    tex_id: GLuint,
    multicolor: bool,
    top: i16,
    left: i16,
    width: i16,
    height: i16,
    uv_bot: f32,
    uv_left: f32,
    uv_width: f32,
    uv_height: f32,
}

#[derive(Debug)]
#[repr(C)]
struct InstanceData {
    x: i16,
    y: i16,
    left: i16,
    top: i16,
    width: i16,
    height: i16,
    uv_left: f32,
    uv_bot: f32,
    uv_width: f32,
    uv_height: f32,
    r: u8,
    g: u8,
    b: u8,
    cell_flags: RenderingGlyphFlags,
}

#[derive(Debug, Default)]
pub struct Batch {
    tex: GLuint,
    instances: Vec<InstanceData>,
}

pub struct TextRenderer {
    program: Program,
    vao: GLuint,
    ebo: GLuint,
    vbo_instance: GLuint,
    atlas: Vec<Atlas>,
    current_atlas: usize,
    active_tex: GLuint,
    batch: Batch,
    size: Size,
    font_key: FontKey,
    cache: HashMap<GlyphKey, Glyph, BuildHasherDefault<FnvHasher>>,
    rasterizer: Rasterizer,
    metrics: Metrics,
    spacing: i16,
}

#[derive(Debug)]
struct Atlas {
    id: GLuint,
    width: i32,
    height: i32,
    row_extent: i32,
    row_baseline: i32,
    row_tallest: i32,
}

impl Batch {
    #[inline]
    pub fn new() -> Self {
        Self {
            tex: 0,
            instances: Vec::with_capacity(BATCH_MAX),
        }
    }

    pub fn add_item(&mut self, x: i16, y: i16, r: u8, g: u8, b: u8, glyph: &Glyph) {
        if self.is_empty() {
            self.tex = glyph.tex_id;
        }

        let mut cell_flags = RenderingGlyphFlags::empty();
        cell_flags.set(RenderingGlyphFlags::COLORED, glyph.multicolor);
        cell_flags.set(RenderingGlyphFlags::WIDE_CHAR, true);

        self.instances.push(InstanceData {
            x,
            y,
            r,
            g,
            b,
            top: glyph.top,
            left: glyph.left,
            width: glyph.width,
            height: glyph.height,
            uv_bot: glyph.uv_bot,
            uv_left: glyph.uv_left,
            uv_width: glyph.uv_width,
            uv_height: glyph.uv_height,
            cell_flags,
        });
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.instances.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.len() * size_of::<InstanceData>()
    }

    pub fn clear(&mut self) {
        self.tex = 0;
        self.instances.clear();
    }
}

fn compute_font_keys(rasterizer: &mut Rasterizer, font: &str, size: Size) -> FontKey {
    rasterizer
        .load_font(
            &FontDesc::new(
                font,
                Style::Description {
                    slant: Slant::Normal,
                    weight: Weight::Normal,
                },
            ),
            size,
        )
        .unwrap()
}

impl TextRenderer {
    pub fn new(
        font: &str,
        font_size: f32,
        screen_size: Vec2f,
        dpr: f64,
    ) -> Result<TextRenderer, ShaderError> {
        let program = Program::new(
            Shader::new(gl::VERTEX_SHADER, VERTEX)?,
            Shader::new(gl::FRAGMENT_SHADER, FRAGMENT)?,
        )?;

        let mut vao: GLuint = 0;
        let mut ebo: GLuint = 0;

        let mut proj_matrix: Mat4 = [0.; 16];
        mat4::ortho(
            &mut proj_matrix,
            0.,
            screen_size.x,
            screen_size.y,
            0.,
            0.,
            1000.,
        );

        let mut vbo_instance: GLuint = 0;

        unsafe {
            gl::UseProgram(program.id);
            gl::UniformMatrix4fv(
                gl::GetUniformLocation(program.id, b"projection\0".as_ptr() as *const _),
                1,
                gl::FALSE,
                proj_matrix.as_ptr(),
            );

            gl::UseProgram(0);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR);
            gl::Enable(gl::MULTISAMPLE);

            gl::DepthMask(gl::FALSE);

            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut ebo);
            gl::GenBuffers(1, &mut vbo_instance);
            gl::BindVertexArray(vao);

            let indices: [u32; 6] = [0, 1, 3, 1, 2, 3];

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (6 * size_of::<u32>()) as isize,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo_instance);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (BATCH_MAX * size_of::<InstanceData>()) as isize,
                ptr::null(),
                gl::STREAM_DRAW,
            );

            let mut index = 0;
            let mut size = 0;

            macro_rules! add_attr {
                ($count:expr, $gl_type:expr, $type:ty) => {
                    gl::VertexAttribPointer(
                        index,
                        $count,
                        $gl_type,
                        gl::FALSE,
                        size_of::<InstanceData>() as i32,
                        size as *const _,
                    );
                    gl::EnableVertexAttribArray(index);
                    gl::VertexAttribDivisor(index, 1);

                    #[allow(unused_assignments)]
                    {
                        size += $count * size_of::<$type>();
                        index += 1;
                    }
                };
            }

            add_attr!(2, gl::UNSIGNED_SHORT, u16);
            add_attr!(4, gl::SHORT, i16);
            add_attr!(4, gl::FLOAT, f32);
            add_attr!(4, gl::UNSIGNED_BYTE, u8);

            gl::UseProgram(0);
            gl::Disable(gl::BLEND);
            gl::Disable(gl::MULTISAMPLE);

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        let size = Size::new(font_size);

        let mut rasterizer = Rasterizer::new(dpr as f32, false).unwrap();
        let font_key = compute_font_keys(&mut rasterizer, font, size);
        let metrics = rasterizer.metrics(font_key, size).unwrap();

        let mut renderer = Self {
            program,
            vao,
            ebo,
            vbo_instance,
            atlas: Vec::new(),
            current_atlas: 0,
            active_tex: 0,
            batch: Batch::new(),
            cache: HashMap::default(),
            rasterizer,
            metrics,
            size,
            font_key,
            spacing: 8,
        };

        let atlas = Atlas::new(ATLAS_SIZE);
        renderer.atlas.push(atlas);

        for i in 32u8..=126u8 {
            renderer.get_glyph(GlyphKey {
                font_key,
                character: i as char,
                size,
            });
        }

        Ok(renderer)
    }

    pub fn draw_char(&mut self, character: char, x: i16, y: i16) {
        let glyph = self.get_glyph(GlyphKey {
            character,
            font_key: self.font_key,
            size: self.size,
        });

        self.batch.add_item(x, y, 255, 255, 255, &glyph);
        self.render_batch();
    }

    pub fn get_length(&mut self, string: &str) -> i16 {
        let width: i16 = string
            .chars()
            .enumerate()
            .map(|(_, character)| {
                let mut w = self
                    .get_glyph(GlyphKey {
                        character,
                        font_key: self.font_key,
                        size: self.size,
                    })
                    .width;

                if w == 0 {
                    w = self.metrics.average_advance as i16;
                }

                w + self.spacing
            })
            .sum();

        width
    }

    pub fn get_height(&self) -> i16 {
        self.metrics.line_height as i16
    }

    pub fn draw_string(&mut self, string: &str, x: i16, y: i16, hex: i32) {
        let mut t_x = x;
        let glyphs = string
            .chars()
            .enumerate()
            .map(|(_, character)| {
                self.get_glyph(GlyphKey {
                    character,
                    font_key: self.font_key,
                    size: self.size,
                })
            })
            .collect::<Vec<_>>();

        for glyph in glyphs {
            let red = ((hex >> 16) & 0xFF) as u8;
            let green = ((hex >> 8) & 0xFF) as u8;
            let blue = (hex & 0xFF) as u8;

            self.batch.add_item(t_x, y, red, green, blue, &glyph);

            if glyph.width <= 0 {
                t_x += self.metrics.average_advance as i16;
            }

            t_x += glyph.width + self.spacing;
        }

        self.render_batch();
    }

    pub fn render_batch(&mut self) {
        unsafe {
            gl::UseProgram(self.program.id);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC1_COLOR, gl::ONE_MINUS_SRC1_COLOR);
            gl::Enable(gl::MULTISAMPLE);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_instance);

            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                self.batch.size() as isize,
                self.batch.instances.as_ptr() as *const _,
            );

            if self.active_tex != self.batch.tex {
                gl::BindTexture(gl::TEXTURE_2D, self.batch.tex);
                self.active_tex = self.batch.tex;
            }

            gl::Uniform2f(
                gl::GetUniformLocation(self.program.id, b"cellDim\0".as_ptr() as *const _),
                self.size.as_f32_pts(),
                self.size.as_f32_pts() * 2.,
            );
            gl::DrawElementsInstanced(
                gl::TRIANGLES,
                6,
                gl::UNSIGNED_INT,
                ptr::null(),
                self.batch.len() as GLsizei,
            );

            gl::UseProgram(0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            gl::Disable(gl::BLEND);
            gl::Disable(gl::MULTISAMPLE);

            self.batch.clear();
        }
    }

    pub fn get_glyph(&mut self, key: GlyphKey) -> Glyph {
        if let Some(glyph) = self.cache.get(&key) {
            return *glyph;
        };

        let glyph = match self.rasterizer.get_glyph(key) {
            Ok(rasterized) => self.load_glyph(rasterized),
            Err(RasterizerError::MissingGlyph(rasterized)) => {
                let missing_key = GlyphKey {
                    character: '\0',
                    ..key
                };
                if let Some(glyph) = self.cache.get(&missing_key) {
                    *glyph
                } else {
                    let glyph = self.load_glyph(rasterized);
                    self.cache.insert(missing_key, glyph);

                    glyph
                }
            }
            Err(_) => self.load_glyph(Default::default()),
        };

        *self.cache.entry(key).or_insert(glyph)
    }

    pub fn load_glyph(&mut self, rasterized: RasterizedGlyph) -> Glyph {
        match self.atlas[self.current_atlas].insert(&rasterized, &mut self.active_tex) {
            Ok(glyph) => glyph,
            Err(AtlasInsertError::Full) => {
                self.current_atlas += 1;
                if self.current_atlas == self.atlas.len() {
                    let new = Atlas::new(ATLAS_SIZE);
                    self.active_tex = 0;
                    self.atlas.push(new);
                }
                self.load_glyph(rasterized)
            }
            Err(AtlasInsertError::GlyphTooLarge) => Glyph {
                tex_id: self.atlas[self.current_atlas].id,
                multicolor: false,
                top: 0,
                left: 0,
                width: 0,
                height: 0,
                uv_bot: 0.,
                uv_left: 0.,
                uv_width: 0.,
                uv_height: 0.,
            },
        }
    }
}

enum AtlasInsertError {
    Full,
    GlyphTooLarge,
}

impl Atlas {
    fn new(size: i32) -> Self {
        let mut id: GLuint = 0;
        unsafe {
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                size,
                size,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        Self {
            id,
            width: size,
            height: size,
            row_extent: 0,
            row_baseline: 0,
            row_tallest: 0,
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.row_extent = 0;
        self.row_baseline = 0;
        self.row_tallest = 0;
    }

    pub fn insert(
        &mut self,
        glyph: &RasterizedGlyph,
        active_tex: &mut u32,
    ) -> Result<Glyph, AtlasInsertError> {
        if glyph.width > self.width || glyph.height > self.height {
            return Err(AtlasInsertError::GlyphTooLarge);
        }

        if !self.room_in_row(glyph) {
            self.advance_row()?;
        }

        if !self.room_in_row(glyph) {
            return Err(AtlasInsertError::Full);
        }

        Ok(self.insert_inner(glyph, active_tex))
    }

    fn insert_inner(&mut self, glyph: &RasterizedGlyph, active_tex: &mut u32) -> Glyph {
        let offset_y = self.row_baseline;
        let offset_x = self.row_extent;
        let height = glyph.height as i32;
        let width = glyph.width as i32;
        let multicolor;

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.id);

            let (format, buffer) = match &glyph.buffer {
                BitmapBuffer::Rgb(buffer) => {
                    multicolor = false;
                    (gl::RGB, buffer)
                }
                BitmapBuffer::Rgba(buffer) => {
                    multicolor = true;
                    (gl::RGBA, buffer)
                }
            };

            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                offset_x,
                offset_y,
                width,
                height,
                format,
                gl::UNSIGNED_BYTE,
                buffer.as_ptr() as *const _,
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);
            *active_tex = 0;
        }

        self.row_extent += width;
        if height > self.row_tallest {
            self.row_tallest = height;
        }

        let uv_bot = offset_y as f32 / self.height as f32;
        let uv_left = offset_x as f32 / self.width as f32;
        let uv_height = height as f32 / self.height as f32;
        let uv_width = width as f32 / self.width as f32;

        Glyph {
            tex_id: self.id,
            multicolor,
            top: glyph.top as i16,
            left: glyph.left as i16,
            width: width as i16,
            height: height as i16,
            uv_bot,
            uv_left,
            uv_width,
            uv_height,
        }
    }

    fn room_in_row(&self, raw: &RasterizedGlyph) -> bool {
        let next_extent = self.row_extent + raw.width as i32;
        let enough_width = next_extent <= self.width;
        let enough_height = (raw.height as i32) < (self.height - self.row_baseline);

        enough_width && enough_height
    }

    fn advance_row(&mut self) -> Result<(), AtlasInsertError> {
        let advance_to = self.row_baseline + self.row_tallest;
        if self.height - advance_to <= 0 {
            return Err(AtlasInsertError::Full);
        }

        self.row_baseline = advance_to;
        self.row_extent = 0;
        self.row_tallest = 0;

        Ok(())
    }
}

impl Drop for Atlas {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
