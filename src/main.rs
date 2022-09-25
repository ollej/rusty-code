#![windows_subsystem = "windows"]

use jsonpath_rust::JsonPathFinder;
use macroquad::prelude::*;
use quad_net::http_request::{HttpError, RequestBuilder};
use quad_url::get_program_parameters;
use rusty_slider::prelude::*;
use std::error;
use std::fmt;
use std::path::PathBuf;
use structopt::StructOpt;

struct Code {
    filename: String,
    sourcecode: String,
}

impl Code {
    fn new(filename: String, sourcecode: String) -> Self {
        Self {
            filename,
            sourcecode,
        }
    }

    fn from_sourcecode(sourcecode: String) -> Self {
        Self {
            filename: "noname.txt".to_string(),
            sourcecode,
        }
    }

    fn language(&self, language_override: Option<String>) -> Option<String> {
        language_override
            .or_else(|| detect_lang::from_path(&self.filename).map(|lang| lang.id().to_string()))
    }

    async fn load(
        gist: Option<String>,
        filename: Option<PathBuf>,
        code: Option<String>,
    ) -> Result<Code> {
        if let Some(content) = code {
            return Ok(Code::from_sourcecode(content));
        }
        if let Some(gist_id) = gist {
            return get_gist_file(gist_id).await;
        }
        let file = Self::get_filename(filename);
        return load_string(&file)
            .await
            .map(|code| Code::new(file, code))
            .map_err(|e| e.into());
    }

    fn get_filename(filename: Option<PathBuf>) -> String {
        filename
            .map(|file| file.to_string_lossy().into_owned())
            .unwrap_or("assets/helloworld.rs".to_string())
    }
}

type Result<T> = std::result::Result<T, CodeError>;

#[derive(Debug)]
enum CodeError {
    FileError(String, FileError),
    GistLoadError(String, HttpError),
    FontError(String, FontError),
    GistParseError(String),
}

impl fmt::Display for CodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            CodeError::FileError(filename, _e) => write!(f, "Couldn't load file: {}", filename),
            CodeError::GistLoadError(gist_id, _e) => {
                write!(f, "Couldn't load Gist with ID: {}", gist_id)
            }
            CodeError::FontError(filename, _e) => write!(f, "Couldn't load font: {}", filename),
            CodeError::GistParseError(message) => write!(f, "Couldn't parse JSON: {}", message),
        }
    }
}

impl error::Error for CodeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match &*self {
            CodeError::FileError(_, ref e) => Some(e),
            CodeError::GistLoadError(_, _e) => None,
            CodeError::FontError(_, ref e) => Some(e),
            CodeError::GistParseError(_) => None,
        }
    }
}

impl From<FileError> for CodeError {
    fn from(err: FileError) -> CodeError {
        CodeError::FileError(err.path.clone(), err)
    }
}

async fn load_gist(gist_id: String) -> Result<String> {
    let path = format!("https://api.github.com/gists/{}", gist_id);
    let mut request = RequestBuilder::new(path.as_str())
        .header("Accept", "application/vnd.github.v3+json")
        .send();
    loop {
        if let Some(result) = request.try_recv() {
            return result.map_err(|e| CodeError::GistLoadError(gist_id, e));
        };
        next_frame().await;
    }
}

fn parse_gist_response(json: String) -> Result<Code> {
    let finder = JsonPathFinder::from_str(&json, "$.files.*['filename', 'content']")
        .map_err(|e| CodeError::GistParseError(e))?;
    let gist = finder.find_slice();
    let gist_filename = gist.first().unwrap().as_str().unwrap().to_string();
    let gist_content = gist.get(1).unwrap().as_str().unwrap().to_string();
    debug!(
        "gist filename:\n{},\ngist_content:\n{}",
        gist_filename, gist_content
    );
    Ok(Code::new(gist_filename, gist_content))
}

async fn get_gist_file(gist_id: String) -> Result<Code> {
    let json = load_gist(gist_id).await?;
    parse_gist_response(json)
}

async fn build_codebox(opt: &CliOptions, theme: &Theme) -> Result<CodeBox> {
    let font_bold = load_ttf_font(&theme.font_bold)
        .await
        .map_err(|e| CodeError::FontError(theme.font_bold.clone(), e))?;
    let font_italic = load_ttf_font(&theme.font_italic)
        .await
        .map_err(|e| CodeError::FontError(theme.font_italic.clone(), e))?;
    let font_code = load_ttf_font(&theme.font_code)
        .await
        .map_err(|e| CodeError::FontError(theme.font_code.clone(), e))?;

    let code = Code::load(opt.gist.clone(), opt.filename.clone(), opt.code.clone()).await?;
    let language = code.language(opt.language.clone());

    let code_box_builder = CodeBoxBuilder::new(theme.clone(), font_code, font_bold, font_italic);

    Ok(code_box_builder.build_draw_box(language, code.sourcecode))
}

fn draw_error_message(message: String, font_size: u16) {
    let text_dim = measure_text(&message, None, font_size, 1.0);
    let xpos = screen_width() / 2. - text_dim.width / 2.;
    let ypos = screen_height() / 2. - text_dim.height / 2.;
    draw_text_ex(
        &message,
        xpos,
        ypos,
        TextParams {
            font_size: font_size,
            ..TextParams::default()
        },
    );
}
#[derive(StructOpt, Debug)]
#[structopt(
    name = "rusty-code",
    about = "A small tool to display sourcecode files"
)]
struct CliOptions {
    /// Code to display, overrides both `filename` and `gist`
    #[structopt(short, long)]
    pub code: Option<String>,
    /// Path to sourcecode file to display [default: assets/helloworld.rs]
    #[structopt(short, long, parse(from_os_str))]
    pub filename: Option<PathBuf>,
    /// Gist id to display, if set, will override `filename` option
    #[structopt(short, long)]
    pub gist: Option<String>,
    /// Language of the code, if empty defaults to file extension.
    #[structopt(short, long)]
    pub language: Option<String>,
    /// Path to theme.json file
    #[structopt(short, long, parse(from_os_str), default_value = "assets/theme.json")]
    pub theme: PathBuf,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Rusty Code".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

/// Binary to display source code with Macroquad
#[macroquad::main(window_conf)]
async fn main() {
    let opt = CliOptions::from_iter(get_program_parameters().iter());
    let theme = Theme::load(opt.theme.clone()).await;

    let codebox_result = build_codebox(&opt, &theme).await;
    if let Err(e) = &codebox_result {
        error!("Encountered an error: {}", e);
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::process::exit(1);
        }
    }

    let render_target = render_target(500, 500);
    render_target.texture.set_filter(FilterMode::Nearest);

    let material = load_material(
        GRADIENT_VERTEX_SHADER,
        GRADIENT_FRAGMENT_SHADER,
        MaterialParams {
            uniforms: vec![("canvasSize".to_owned(), UniformType::Float2)],
            ..Default::default()
        },
    )
    .unwrap();
    loop {
        #[cfg(not(target_arch = "wasm32"))]
        if is_key_pressed(KeyCode::Q) | is_key_pressed(KeyCode::Escape) {
            break;
        }

        // 0..100, 0..100 camera
        set_camera(&Camera2D {
            zoom: vec2(0.01, 0.01),
            target: vec2(0.0, 0.0),
            render_target: Some(render_target),
            ..Default::default()
        });

        // drawing to the screen

        set_default_camera();

        clear_background(WHITE);
        gl_use_material(material);
        material.set_uniform("canvasSize", (screen_width(), screen_height()));
        draw_texture_ex(
            render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        gl_use_default_material();

        //clear_background(theme.background_color);
        match &codebox_result {
            Ok(codebox) => {
                let xpos = screen_width() / 2. - codebox.width_with_padding() as f32 / 2.;
                let ypos = screen_height() / 2. - codebox.height_with_padding() as f32 / 2.;
                codebox.draw(xpos, ypos);
            }
            Err(e) => {
                draw_error_message(e.to_string(), theme.font_size_text as u16);
            }
        };

        next_frame().await
    }
}

const GRADIENT_FRAGMENT_SHADER: &'static str = r#"#version 100
precision lowp float;
uniform vec2 canvasSize;
uniform sampler2D Texture;

void main() {
    vec2 coord = gl_FragCoord.xy/canvasSize.xy;
    gl_FragColor = vec4(coord.x, coord.y, 1.-coord.x, 1);
}
"#;

const GRADIENT_VERTEX_SHADER: &'static str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
}
";
