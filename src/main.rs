use macroquad::prelude::*;
use rusty_slider::prelude::*;
use std::env;
use std::path::PathBuf;
#[cfg(not(debug_assertions))]
use std::process;

fn window_conf() -> Conf {
    Conf {
        window_title: "Rusty Code".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

fn get_filename() -> String {
    env::args().nth(1).unwrap_or_else(|| default_filename())
}

#[cfg(debug_assertions)]
fn default_filename() -> String {
    "assets/helloworld.rs".to_string()
}

#[cfg(not(debug_assertions))]
fn default_filename() -> String {
    explain_usage()
}

#[cfg(not(debug_assertions))]
fn explain_usage() -> ! {
    println!("Display a GIF file.\n\nUsage: quad-gif <file>");
    process::exit(1)
}

/// Binary to display source code with Macroquad
#[macroquad::main(window_conf)]
async fn main() {
    let filename = get_filename();
    let language = detect_lang::from_path(filename.clone()).map(|lang| lang.id().to_string());
    let source_code = load_string(filename.as_str())
        .await
        .expect("Couldn't find sourcecode file!");
    let theme = Theme::load(PathBuf::from("assets/theme.json".to_string())).await;
    let font_bold = load_ttf_font(&theme.font_bold)
        .await
        .expect("Couldn't load font");
    let font_italic = load_ttf_font(&theme.font_italic)
        .await
        .expect("Couldn't load font");
    let font_code = load_ttf_font(&theme.font_code)
        .await
        .expect("Couldn't load font");
    let code_box_builder = CodeBoxBuilder::new(theme.clone(), font_code, font_bold, font_italic);
    let codebox = code_box_builder.build_draw_box(language, source_code);

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
