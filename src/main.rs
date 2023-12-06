
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = &args[1];
    let paths = std::fs::read_dir(path.to_owned() + "/text-src").expect("text-src should exist");
    let template = std::fs::read_to_string(path.to_owned() + "/text-src/template.html").expect("template.html should exist");
    

}
