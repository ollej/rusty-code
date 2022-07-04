use jsonpath_rust::JsonPathFinder;
use macroquad::prelude::*;
use quad_net::http_request::RequestBuilder;
use quad_url::get_program_parameters;
use rusty_slider::prelude::*;
use std::error::Error;
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
    ) -> Result<Code, Box<dyn Error>> {
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

fn window_conf() -> Conf {
    Conf {
        window_title: "Rusty Code".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

async fn load_gist(gist_id: String) -> Result<String, Box<dyn Error>> {
    let path = format!("https://api.github.com/gists/{}", gist_id);
    let mut request = RequestBuilder::new(path.as_str())
        .header("Accept", "application/vnd.github.v3+json")
        .send();
    loop {
        if let Some(result) = request.try_recv() {
            return result.map_err(|e| e.to_string().into());
        };
        next_frame().await;
    }
}

fn parse_gist_response(json: String) -> Result<Code, Box<dyn Error>> {
    let finder = JsonPathFinder::from_str(&json, "$.files.*['filename', 'content']")?;
    let gist = finder.find_slice();
    let gist_filename = gist.first().unwrap().as_str().unwrap().to_string();
    let gist_content = gist.get(1).unwrap().as_str().unwrap().to_string();
    debug!(
        "gist filename:\n{},\ngist_content:\n{}",
        gist_filename, gist_content
    );
    Ok(Code::new(gist_filename, gist_content))
}

async fn get_gist_file(gist_id: String) -> Result<Code, Box<dyn Error>> {
    let json = load_gist(gist_id).await?;
    parse_gist_response(json)
}

async fn build_codebox(opt: &CliOptions, theme: &Theme) -> Result<TextBox, Box<dyn Error>> {
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
    let ypos = screen_height() / 2. - text_dim.height / 2. + text_dim.offset_y;
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

    loop {
        #[cfg(not(target_arch = "wasm32"))]
        if is_key_pressed(KeyCode::Q) | is_key_pressed(KeyCode::Escape) {
            break;
        }

        clear_background(theme.background_color);
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
