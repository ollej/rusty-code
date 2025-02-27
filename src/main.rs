#![windows_subsystem = "windows"]

use rusty_slider::prelude::*;
use std::{error, fmt, path::PathBuf};
use {
    clap::Parser,
    jsonpath_rust::JsonPathFinder,
    macroquad::prelude::*,
    quad_net::http_request::{HttpError, RequestBuilder},
    quad_url::get_program_parameters,
};

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
        load_string(&file)
            .await
            .map(|code| Code::new(file, code))
            .map_err(|e| e.into())
    }

    fn get_filename(filename: Option<PathBuf>) -> String {
        filename
            .map(|file| file.to_string_lossy().into_owned())
            .unwrap_or_else(|| "assets/helloworld.rs".to_string())
    }
}

type Result<T> = std::result::Result<T, CodeError>;

#[derive(Debug)]
enum CodeError {
    File(String, macroquad::miniquad::fs::Error),
    GistLoad(String, HttpError),
    Font(String),
    GistParse(String),
    Macroquad(macroquad::Error),
}

impl fmt::Display for CodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CodeError::File(filename, _e) => write!(f, "Couldn't load file: {}", filename),
            CodeError::GistLoad(gist_id, _e) => {
                write!(f, "Couldn't load Gist with ID: {}", gist_id)
            }
            CodeError::Font(error) => write!(f, "Couldn't load font: {:?}", error),
            CodeError::GistParse(message) => write!(f, "Couldn't parse JSON: {}", message),
            CodeError::Macroquad(err) => write!(f, "Macroquad error: {:?}", err),
        }
    }
}

impl error::Error for CodeError {}

impl From<macroquad::Error> for CodeError {
    fn from(err: macroquad::Error) -> CodeError {
        match err {
            macroquad::Error::FontError(msg) => CodeError::Font(msg.to_string()),
            macroquad::Error::FileError { kind, path } => CodeError::File(path.clone(), kind),
            macroquad::Error::ShaderError(_) => CodeError::Macroquad(err),
            macroquad::Error::ImageError(_) => CodeError::Macroquad(err),
            macroquad::Error::UnknownError(_) => CodeError::Macroquad(err),
        }
    }
}

async fn load_gist(gist_id: String) -> Result<String> {
    let path = format!("https://api.github.com/gists/{}", gist_id);
    let mut request = RequestBuilder::new(path.as_str())
        .header("Accept", "application/vnd.github.v3+json")
        .send();
    loop {
        if let Some(result) = request.try_recv() {
            return result.map_err(|e| CodeError::GistLoad(gist_id, e));
        };
        next_frame().await;
    }
}

fn parse_gist_response(json: String) -> Result<Code> {
    let finder = JsonPathFinder::from_str(&json, "$.files.*['filename', 'content']")
        .map_err(CodeError::GistParse)?;
    let gist = finder.find_slice();
    let gist_filename = gist
        .first()
        .ok_or_else(|| CodeError::GistParse("Filename missing".to_string()))?
        .clone()
        .to_data()
        .as_str()
        .ok_or_else(|| CodeError::GistParse("Couldn't parse filename".to_string()))?
        .to_string();
    let gist_content = gist
        .get(1)
        .ok_or_else(|| CodeError::GistParse("Content missing".to_string()))?
        .clone()
        .to_data()
        .as_str()
        .ok_or_else(|| CodeError::GistParse("Couldn't parse filename".to_string()))?
        .to_string();
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
    let font_bold = load_ttf_font(&theme.font_bold).await?;
    let font_italic = load_ttf_font(&theme.font_italic).await?;
    let font_code = load_ttf_font(&theme.font_code).await?;

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
            font_size,
            ..TextParams::default()
        },
    );
}
#[derive(Parser, Debug)]
#[command(
    name = "rusty-code",
    about = "A small tool to display sourcecode files"
)]
struct CliOptions {
    /// Code to display, overrides both `filename` and `gist`
    #[arg(short, long)]
    pub code: Option<String>,
    /// Path to sourcecode file to display [default: assets/helloworld.rs]
    #[arg(short, long)]
    pub filename: Option<PathBuf>,
    /// Gist id to display, if set, will override `filename` option
    #[arg(short, long)]
    pub gist: Option<String>,
    /// Language of the code, if empty defaults to file extension.
    #[arg(short, long)]
    pub language: Option<String>,
    /// Path to theme.json file
    #[arg(short, long, default_value = "assets/theme.json")]
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
    let opt = CliOptions::parse_from(get_program_parameters().iter());
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
        ShaderSource::Glsl {
            vertex: GRADIENT_VERTEX_SHADER,
            fragment: GRADIENT_FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![UniformDesc::new("canvasSize", UniformType::Float2)],
            ..Default::default()
        },
    )
    .expect("Couldn't load material");
    loop {
        #[cfg(not(target_arch = "wasm32"))]
        if is_key_pressed(KeyCode::Q) | is_key_pressed(KeyCode::Escape) {
            break;
        }

        // 0..100, 0..100 camera
        set_camera(&Camera2D {
            zoom: vec2(0.01, 0.01),
            target: vec2(0.0, 0.0),
            render_target: Some(render_target.clone()),
            ..Default::default()
        });

        // drawing to the screen

        set_default_camera();

        clear_background(WHITE);
        gl_use_material(&material);
        material.set_uniform("canvasSize", (screen_width(), screen_height()));
        draw_texture_ex(
            &render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        gl_use_default_material();

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

const GRADIENT_FRAGMENT_SHADER: &str = r#"#version 100
precision lowp float;
uniform vec2 canvasSize;
uniform sampler2D Texture;

void main() {
    vec2 coord = gl_FragCoord.xy/canvasSize.xy;
    gl_FragColor = vec4(coord.x, coord.y, 1.-coord.x, 1);
}
"#;

const GRADIENT_VERTEX_SHADER: &str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
}
";
