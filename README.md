# pagebuild

This is a static site generator written in rust.

## How to run:

Run this by building the executable and running:<br> 
`pagebuild "path_to_text-src" "path_to_html_files"`<br>
Flags: `--verbose` for full output

What this needs: 
- A directory that contains the website
- In there, a folder "text-src", containing:
  - your desired pages as markdown files
  - a template.html file. The text {{Content}} in the template file will be replaced with your content, gererated from the markdown files
  - optionally a components.html file. The format is documented in `examples/text-src/components.html/`. Components allow easy reuse of specific html or markdown snippets.

Your .html files will be saved to `path_to_html_files/*.html`, so resources like stylesheets are referenced from there and not from the `/text-src/` folder.
The example website outputs a few errors regarding components intentionally.
