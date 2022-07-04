use jsonpath_rust::JsonPathFinder;
use macroquad::prelude::*;
use quad_net::http_request::{HttpError, RequestBuilder};
use quad_url::get_program_parameters;
use rusty_slider::prelude::*;
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

    async fn load_sourcecode(
        gist: Option<String>,
        filename: Option<PathBuf>,
        code: Option<String>,
    ) -> Result<Code, String> {
        if let Some(content) = code {
            return Ok(Code::from_sourcecode(content));
        }
        if let Some(gist_id) = gist {
            return get_gist_file(gist_id).await;
        }
        let file = Self::get_filename(filename);
        return match load_string(&file).await {
            Ok(code) => Ok(Code::new(file, code)),
            Err(_) => Err(format!("Couldn't load sourcecode from file '{}'", file)),
        };
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

async fn load_gist(gist_id: String) -> Result<String, HttpError> {
    let path = format!("https://api.github.com/gists/{}", gist_id);
    let mut request = RequestBuilder::new(path.as_str())
        .header("Accept", "application/vnd.github.v3+json")
        .send();
    loop {
        if let Some(result) = request.try_recv() {
            return result;
        };
        next_frame().await;
    }
}

fn parse_gist_response(json: String) -> Code {
    let finder = JsonPathFinder::from_str(&json, "$.files.*['filename', 'content']")
        .expect("Couldn't parse Gist JSON!");
    let gist = finder.find_slice();
    let gist_filename = gist.first().unwrap().as_str().unwrap().to_string();
    let gist_content = gist.get(1).unwrap().as_str().unwrap().to_string();
    debug!(
        "gist filename:\n{},\ngist_content:\n{}",
        gist_filename, gist_content
    );
    Code::new(gist_filename, gist_content)
}

async fn get_gist_file(gist_id: String) -> Result<Code, String> {
    match load_gist(gist_id).await {
        Ok(json) => Ok(parse_gist_response(json)),
        Err(_) => Err("Couldn't load gist!".to_string()),
    }
}

async fn build_codebox(opt: &CliOptions, theme: &Theme) -> TextBox {
    let font_bold = load_ttf_font(&theme.font_bold)
        .await
        .expect("Couldn't load font");
    let font_italic = load_ttf_font(&theme.font_italic)
        .await
        .expect("Couldn't load font");
    let font_code = load_ttf_font(&theme.font_code)
        .await
        .expect("Couldn't load font");

    let code = Code::load_sourcecode(opt.gist.clone(), opt.filename.clone(), opt.code.clone())
        .await
        .expect("Couldn't load sourcecode!");
    let language = code.language(opt.language.clone());

    let code_box_builder = CodeBoxBuilder::new(theme.clone(), font_code, font_bold, font_italic);

    code_box_builder.build_draw_box(language, code.sourcecode)
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

    let codebox = build_codebox(&opt, &theme).await;

    loop {
        #[cfg(not(target_arch = "wasm32"))]
        if is_key_pressed(KeyCode::Q) | is_key_pressed(KeyCode::Escape) {
            break;
        }

        clear_background(theme.background_color);
        let xpos = screen_width() / 2. - codebox.width_with_padding() as f32 / 2.;
        let ypos = screen_height() / 2. - codebox.height_with_padding() as f32 / 2.;
        codebox.draw(xpos, ypos);

        next_frame().await
    }
}
