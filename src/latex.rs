// TODO ADD LOGGING
// TODO EXPLAIN WHY WE HAVE TO DO THIS

use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tempfile::tempdir;
use which::which;

fn require_executable(name: &str) {
    which(name).expect(&format!("Executable {} not found. Please install it and make sure it is in your PATH.", name));
}


fn create_latex_file(formula: &str, filename: &Path) -> io::Result<()> {
    let mut file = File::create(filename)?;
    writeln!(file, "\\documentclass{{standalone}}")?;
    writeln!(file, "\\usepackage{{amsmath}}")?;
    writeln!(file, "\\begin{{document}}")?;
    writeln!(file, "${}$", formula)?;
    writeln!(file, "\\end{{document}}")?;
    Ok(())
}

fn compile_latex_file(input_file: &Path, output_file: &Path) -> io::Result<()> {
    let mut cmd = Command::new("pdflatex");
    cmd.current_dir(input_file.parent().unwrap());
    cmd.arg(input_file);

    let output = cmd.output()?;
    println!("pdflatex output: {}", String::from_utf8_lossy(&output.stdout));
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "pdflatex failed with code {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }

    let pdf_file = input_file.with_extension("pdf");
    println!("pdf file: {:?}", pdf_file);
    let mut cmd = Command::new("pdfcrop");
    cmd.arg(pdf_file)
        .arg(output_file);
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "pdfcrop failed with code {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }

    Ok(())
}

fn convert_pdf_to_svg(input_file: &Path, output_file: &Path) -> io::Result<()> {
    let mut cmd = Command::new("dvisvgm");
    cmd.arg("--pdf")
        .arg(input_file)
        .arg("-o")
        .arg(output_file);

    let output = cmd.output()?;
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "dvisvgm failed with code {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }

    Ok(())
}

use resvg::usvg_text_layout::{fontdb, TreeTextToPath};

/* Based on https://github.com/RazrFalcon/resvg/blob/master/examples/minimal.rs */
// TODO fix all that fucked up full paths
// TODO: ALSO FIX ERROR HANDLING HERE
fn convert_svg_to_png(input_file: &Path, output_file: &Path) -> io::Result<()> {
    let mut opt = resvg::usvg::Options::default();
    // Get file's absolute directory.
    opt.resources_dir = std::fs::canonicalize(input_file)
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();

    let svg_data = std::fs::read(input_file).unwrap();
    let mut tree = resvg::usvg::Tree::from_data(&svg_data, &opt).unwrap();
    tree.convert_text(&fontdb);

    let pixmap_size = tree.size.to_screen_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
    resvg::render(
        &tree,
        resvg::usvg::FitTo::Original,
        resvg::tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();
    pixmap.save_png(output_file).unwrap();
    Ok(())
}


pub fn render_formula(formula: &str, out_path: &Path) -> io::Result<()> {
    let tmp_dir = tempdir()?;
    // TODO REPLACE WITH UUID AS NAME
    let tmp_file = tmp_dir.path().join("formula.tex");
    let pdf_file = tmp_dir.path().join("formula.pdf");
    let svg_file = tmp_dir.path().join("formula.svg");
    // TODO Check if the png extension is there
    let png_file = out_path;

    create_latex_file(formula, &tmp_file)?;
    compile_latex_file(&tmp_file, &pdf_file)?;
    convert_pdf_to_svg(&pdf_file, &svg_file)?;
    convert_svg_to_png(&svg_file, &png_file)?;

    Ok(())
}
