fn main() {
    let args: Vec<String> = std::env::args().collect();
    let verbose = args.contains(&"--verbose".to_string());
    let path = &args[1];
    println!("Building {}...", path);
    let paths = std::fs::read_dir(path.to_owned() + "\\text-src").expect("text-src should exist");
    let template = std::fs::read_to_string(path.to_owned() + "/text-src/template.html")
        .expect("template.html should exist");
    let components = std::fs::read_to_string(path.to_owned() + "/text-src/components.html").ok();

    let mut count = 0;
    // Iterate over markdown files
    for md_path in paths.filter(|x| {
        x.as_ref()
            .expect("path should exist")
            .file_name()
            .to_str()
            .expect("string should exist")
            .contains(".md")
    }) {
        match md_path {
            Ok(md_path) => {
                let name = md_path.file_name();
                let name_hum = name
                    .to_str()
                    .expect("should be valid unicode")
                    .split(".md")
                    .next()
                    .expect("should have .md");
                if verbose {
                    println!("Markdown file: {}", name_hum);
                }

                let file_content =
                    std::fs::read_to_string(md_path.path()).expect("file should exist");
                if verbose {
                    println!("Markdown: \n{}", file_content);
                }

                let md_html = md_to_html(file_content, &components);
                if verbose {
                    println!("Generated HTML: \n{}", md_html);
                }

                // Set title
                let mut html_file = template.replace("{{title}}", name_hum);
                // Insert formatted content
                html_file = html_file.replace("{{content}}", &md_html);
                std::fs::write(path.to_string() + "\\" + name_hum + ".html", html_file)
                    .expect("should be able to write to file");
                count += 1;
            }
            Err(_) => {}
        }
    }

    println!("Complete! Build {} file(s)", count);
}

#[derive(PartialEq)]
enum HtmlSection {
    Heading1,
    Heading2,
    Paragraph,
    UnorderedList,
    Quote,
    Image,
    None,
}

fn md_to_html(content: String, components: &Option<String>) -> String {
    let mut html = String::new();
    let mut old_section = HtmlSection::None;
    //let mut is_paragraph = false;
    //let mut is_list = false;
    //let mut is_quote = false;
    let mut ind = 1;

    // Convert lines
    for line in content.lines() {
        if line.contains("{{component: ") {
            let component = populate_component(line, components);
            html = html + &component + "\n";
            continue;
        }

        // Add links to line
        let line = &md_links(line);
        let mut new_section = HtmlSection::None;

        // Get current section
        if line.contains("# ") && line.chars().next() == Some('#')  {
            // Heading1
            new_section = HtmlSection::Heading1;
        } else if line.contains("## ") && line.chars().next() == Some('#')  {
            // Heading2
            new_section = HtmlSection::Heading2;
        } else if line.contains("- ") && line.chars().next() == Some('-')  {
            // List
            new_section = HtmlSection::UnorderedList;
        } else if line.contains("> ") && line.chars().next() == Some('>')  {
            // Quote
            new_section = HtmlSection::Quote;
        } else if line.contains("![") && line.chars().next() == Some('!')  {
            // Image
            new_section = HtmlSection::Image;
        } else {
            // Paragraph
            if line != "" {
                new_section = HtmlSection::Paragraph;
            }
        }

        // Close old section if necessary:
        if old_section != new_section {
            match old_section {
                HtmlSection::Paragraph => {
                    ind -= 1;
                    html = html + &indent(ind) + "</p>\n";
                },
                HtmlSection::UnorderedList => {
                    ind -= 1;
                    html = html + &indent(ind) + "</ul>\n";
                },
                HtmlSection::Quote => {
                    ind -= 1;
                    html = html + &indent(ind) + "</p>\n";
                },
                _ => {}
            }
        }

        // Add content: 
        // Open new section if necessary
        if old_section != new_section {
            match new_section {
                HtmlSection::Paragraph => {
                    html = html + &indent(ind) + "<p>\n";
                    ind += 1;
                },
                HtmlSection::UnorderedList => {
                    html = html + &indent(ind) + "<ul>\n";
                    ind += 1;
                },
                HtmlSection::Quote => {
                    html = html + &indent(ind) + "<p class=\"quote\">\n";
                    ind += 1;
                },
                _ => {}
            }
        }
        // Add content:
        match new_section {
            HtmlSection::Paragraph => {
                if line != "" {
                    html = html + &indent(ind) + line + "<br>\n"
                }
            },
            HtmlSection::Heading1 => {
                html = html + &indent(ind) + "<h1>" + &line.replace("# ", "") + "</h1>\n";
            },
            HtmlSection::Heading2 => {
                html = html + &indent(ind) + "<h2>" + &line.replace("## ", "") + "</h2>\n";
            },
            HtmlSection::UnorderedList => {
                html = html + &indent(ind) + "<li>" + &line.replace("- ", "") + "</li>\n";
            },
            HtmlSection::Quote => {
                html = html + &indent(ind) + &line.replace("> ", "") + "<br>\n";
            },
            HtmlSection::Image => {
                let (name, dest) = md_img(line);
                html = html + &indent(ind) + "<img src=\"" + &dest + "\" alt=\"" + &name + "\">\n"
            }
            _ => {}
        }

        old_section = new_section;
    }

    // Close if needed
    match old_section {
        HtmlSection::Paragraph => {
            ind -= 1;
            html = html + &indent(ind) + "</p>\n";
        },
        HtmlSection::UnorderedList => {
            ind -= 1;
            html = html + &indent(ind) + "</ul>\n";
        },
        HtmlSection::Quote => {
            ind -= 1;
            html = html + &indent(ind) + "</p>\n";
        },
        _ => {}
    }

    html
}

fn indent(i: u32) -> String {
    let mut space = String::new();
    for _ in 0..i {
        space += "    "
    }
    space
}

fn md_links(line: &str) -> String {
    let mut line_new = String::new();
    let mut is_link = false;
    let mut link_name = String::new();
    let mut is_link_name = false;
    let mut link_dest = String::new();
    let mut is_link_dest = false;
    let mut last_ch = ' ';
    for (_i, ch) in line.chars().enumerate() {
        if is_link_name && ch != ']' {
            link_name.push(ch);
        }
        if ch == '[' && (last_ch != '\\' && last_ch != '!') {
            is_link_name = true;
            is_link = true;
        }
        if ch == ']' && is_link_name {
            is_link_dest = true;
            is_link_name = false;
        }
        if is_link_dest && (ch != '(' && ch != ')' && ch != ']') {
            link_dest.push(ch);
        }

        if !is_link {
            line_new.push(ch);
        }

        if is_link_dest && ch == ')' {
            line_new = line_new + "<a href=\"" + &link_dest + "\">" + &link_name + "</a>";

            is_link = false;
            is_link_dest = false;
            link_name = String::new();
            link_dest = String::new();
        }

        if is_link_name && ch == ']' {
            is_link_name = false;
        }

        last_ch = ch;
    }
    line_new
}

fn md_img(line: &str) -> (String, String) {
    let vec_line: Vec<&str> = line.split("](").collect();
    let name = vec_line[0].replace("![", "");
    let dest = vec_line[1].split(")").next().expect("should have a destination").to_string();
    (name, dest)
}

fn populate_component(line: &str, components: &Option<String>) -> String {
    let new = line.replace("{{component:", "");
    let comp_name = new.split("}}").next().expect("component should not be empty").replace(" ", "");

    match components {
        Some(components) => {
            let component_first = components.split_once(&comp_name);

            match component_first {
                Some((_, component)) => {
                    component.split("</end>").nth(0).expect("component should exist").to_string()
                }
                None => {
                    println!("Warning! Component {} is missing", comp_name);
                    String::new()
                }
            }
        },
        None => {
            println!("Warning! Components are missing");
            String::new()
        }
    }

    
}