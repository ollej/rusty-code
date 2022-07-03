# Rusty Code

Display source code on screen using Macroquad.

![Screenshot](https://ollej.github.io/rusty-code/assets/rusty-code.png)

## Demo

The application also runs in the browser:

[Web demo](https://ollej.github.io/rusty-code/demo/index.html)

## Usage

Run from command line to display sourcecode syntax highlighted on screen using
the Macroquad game library.

```
rusty-code 0.3.0
A small tool to display sourcecode files

USAGE:
    rusty-code [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --code <code>            Code to display, overrides both `filename` and `gist`
    -f, --filename <filename>    Path to sourcecode file to display [default: assets/helloworld.rs]
    -g, --gist <gist>            Gist id to display, if set, will override `filename` option
    -t, --theme <theme>          Path to theme.json file [default: assets/theme.json]
```

## License

Copyright 2022 Olle Wreede, released under the MIT License.
