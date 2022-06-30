use jsonpath_rust::JsonPathFinder;
use macroquad::prelude::*;
use quad_net::http_request::{HttpError, RequestBuilder};
use quad_url::get_program_parameters;
use rusty_slider::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

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

fn parse_gist_response(json: String) -> (String, String) {
    let finder = JsonPathFinder::from_str(&json, "$.files.*['filename', 'content']")
        .expect("Couldn't parse Gist JSON!");
    let gist = finder.find_slice();
    let gist_filename = gist.first().unwrap().as_str().unwrap().to_string();
    let gist_content = gist.get(1).unwrap().as_str().unwrap().to_string();
    debug!(
        "gist filename:\n{},\ngist_content:\n{}",
        gist_filename, gist_content
    );
    (gist_filename, gist_content)
}

async fn get_gist_file(gist_id: String) -> (String, String) {
    let json = load_gist(gist_id).await.expect("Couldn't load gist!");
    parse_gist_response(json)
}

async fn load_sourcecode(gist: Option<String>, filename: PathBuf) -> (String, String) {
    match gist {
        Some(gist_id) => get_gist_file(gist_id).await,
        None => {
            let filename = filename.to_string_lossy().into_owned();
            (
                filename.clone(),
                load_string(&filename)
                    .await
                    .expect("Couldn't find sourcecode file!"),
            )
        }
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

    let (filename, source_code) = load_sourcecode(opt.gist.clone(), opt.filename.clone()).await;
    let language = detect_lang::from_path(filename.clone()).map(|lang| lang.id().to_string());

    let code_box_builder = CodeBoxBuilder::new(theme.clone(), font_code, font_bold, font_italic);

    code_box_builder.build_draw_box(language, source_code)
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "rusty-code",
    about = "A small tool to display sourcecode files"
)]
struct CliOptions {
    /// Path to sourcecode file to display
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = "assets/helloworld.rs"
    )]
    pub filename: PathBuf,
    /// Gist id to display, if set, will override `filename` option
    #[structopt(short, long)]
    pub gist: Option<String>,
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
