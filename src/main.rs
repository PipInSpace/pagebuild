use std::collections::HashMap;

// How many components can be in a line. Needed in case of recursively defined components
const MAX_COMPONENT_DEPTH: u32 = 10;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let verbose = args.contains(&"--verbose".to_string());
    let path = &args[1];

    // Open Files
    println!("🟢 Building {}...", path);
    let paths = std::fs::read_dir(path.to_owned() + "\\text-src");
    let paths = match paths {
        Ok(paths) => paths,
        Err(_) => {
            println!("🔴 WARNING! ./{} does not exist. Aborting.", path);
            std::process::exit(0);
        }
    };

    let template = match std::fs::read_to_string(path.to_owned() + "/text-src/template.html") {
        Ok(string) => string,
        Err(_) => {
            println!("🔴 WARNING! HTML template at ./{}/text-src/template.html does not exist. Aborting.", path);
            std::process::exit(0);
        }
    };
    let components_string =
        std::fs::read_to_string(path.to_owned() + "/text-src/components.html").ok();

    // Html component hashmap
    let mut components: HashMap<String, String> = HashMap::new();
    if let Some(component_string) = components_string {
        parse_components(component_string, &mut components);
    } else {
        println!("🔴 WARNING! components.html is missing.")
    }

    let mut count = 0;
    let mut file_names: Vec<String> = vec![];
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
                    println!("🟠 Markdown: \n{}", file_content);
                }

                let content_populated = populate_components(file_content, &components, verbose);

                let parse = pulldown_cmark::Parser::new(&content_populated);
                let parse = parse.map(|event| match event {
                    pulldown_cmark::Event::SoftBreak => pulldown_cmark::Event::HardBreak,
                    _ => event,
                });

                let mut md_html = String::new();
                pulldown_cmark::html::push_html(&mut md_html, parse);
                //let md_html = md_to_html(file_content, &components);
                if verbose {
                    println!("🟠 Generated HTML: \n{}", md_html);
                }

                // Set title
                let mut html_file = template.replace("{{title}}", name_hum);
                // Insert formatted content
                html_file = html_file.replace("{{content}}", &md_html);
                
                // Write to disk. File names are lowercase and replace spaces with '-'
                std::fs::write(path.to_string() + "\\" + &name_hum.replace(" ", "-").to_lowercase() + ".html", html_file)
                    .expect("should be able to write to file");
                file_names.push(name_hum.replace(" ", "-").to_lowercase());
                count += 1;
            }
            Err(_) => {}
        }
    }

    println!("🟢 Complete! Build {} file(s)", count);
    for name in file_names {
        println!("    - {}.html", name)
    }
}

fn parse_components(component_string: String, component_map: &mut HashMap<String, String>) {
    let mut comp_name = String::new();
    let mut comp = String::new();
    let mut is_comp = false;

    for line in component_string.lines() {
        if line.starts_with("{{") && line.chars().nth(2).expect("should be char") != '/' {
            is_comp = true;
        } else if line.starts_with("{{/") {
            is_comp = false;
        }

        if line.starts_with("{{") && is_comp {
            // Start of component
            comp_name = line.replace(['{', '}'], "");
        } else if line.starts_with("{{/") && !is_comp {
            // End of component, insert into map
            component_map.insert(comp_name.clone(), comp.clone());
            comp_name = String::new();
            comp = String::new();
        } else if is_comp {
            // Part of component
            comp = comp + line + "\n";
        }
    }
}

fn populate_components(content: String, components: &HashMap<String, String>, verbose: bool) -> String {
    let mut new_content = String::new();

    for line in content.lines() {
        if line.contains("{{component:") {
            if verbose {
                println!("🟠 Populating component(s): {}", line);
            }
            new_content = new_content + &comp_line(line, components, 0) + "\n"
        } else {
            new_content = new_content + line + "\n";
        }
    }

    new_content
}

// Recursively generate components
fn comp_line(line: &str, components: &HashMap<String, String>, depth: u32) -> String {
    let depth = depth + 1;
    if depth > MAX_COMPONENT_DEPTH {
        println!("🔴 WARNING! Maximum component depth reached. Is a component recursive?");
        return line.to_string();
    }
    if line.contains("{{component:") {
        let mut new_line = String::new();
        let split = line.split("{{component:").nth(1).expect("should be string");
        let name = split
            .split("}}")
            .nth(0)
            .expect("should be string")
            .replace(" ", "");

        let comp = components.get(&name);
        match comp {
            Some(comp) => {
                new_line = new_line
                    + &line.replace(
                        &("{{component:".to_string()
                            + split.split("}}").nth(0).expect("should be string")
                            + "}}"),
                        &comp,
                    );
                    new_line = comp_line(&new_line, components, depth);
                    return new_line;
            }
            None => {
                println!("🔴 WARNING! Component {} missing.", name);
                // Clear component
                new_line = new_line
                    + &line.replace(
                        &("{{component:".to_string()
                            + split.split("}}").nth(0).expect("should be string")
                            + "}}"),
                        "",
                    )
                    + "\n";
                new_line = comp_line(&new_line, components, depth);
                return new_line;
            }
        }
    } else {
        return line.to_string();
    }
}
