# Rusty Code

Display source code on screen using Macroquad, built with Rust.

![Screenshot](https://ollej.github.io/rusty-code/assets/rusty-code.png)

## Demo

The application also runs in the browser:

[Web demo](https://ollej.github.io/rusty-code/demo/index.html)

### Examples

Display a Gist by adding the Gist id as the `gist` query parameter:

[https://ollej.github.io/rusty-code/demo/index.html?gist=7834a1320cbc1bcfb50304f51c19e618](https://ollej.github.io/rusty-code/demo/index.html?gist=7834a1320cbc1bcfb50304f51c19e618)

Show URL encoded sourcecode with the `code` parameter, also use `language`
parameter to set language for syntax highlighting:

[https://ollej.github.io/rusty-code/demo/index.html?language=rust&code=fn%20main%28%29%20%7B%0A%20%20%20%20println%21%28%22Hello%20World%21%22%29%3B%0A%7D](https://ollej.github.io/rusty-code/demo/index.html?language=rust&code=fn%20main%28%29%20%7B%0A%20%20%20%20println%21%28%22Hello%20World%21%22%29%3B%0A%7D)

### Open a Gist

Enter a Gist id:

<form action="https://ollej.github.io/rusty-code/demo/index.html" method="get">
<input type="text" name="gist">
<input type="submit" value="Display gist">
</form><br>

### Show sourcecode

Enter sourcecode:

<form action="https://ollej.github.io/rusty-code/demo/index.html" method="get">
<select name="language">
<option value="c">C</option>
<option value="cpp">C++</option>
<option value="go">Go</option>
<option value="java">Java</option>
<option value="js">Javascript</option>
<option value="perl">Perl</option>
<option value="python">Python</option>
<option value="ruby">Ruby</option>
<option value="rust" selected>Rust</option>
</select><br>
<textarea name="code" rows="5" cols="60"></textarea><br>
<input type="submit" value="Display code">
</form><br>

## Usage

Run from command line to display sourcecode syntax highlighted on screen using
the Macroquad game library.

```
rusty-code 0.5.0
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
    -l, --language <language>    Language of the code, if empty defaults to file extension
    -t, --theme <theme>          Path to theme.json file [default: assets/theme.json]
```

## License

Copyright 2022 Olle Wreede, released under the MIT License.
