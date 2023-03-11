// TODO ADD LOGGING
// TODO EXPLAIN WHY WE HAVE TO DO THIS

use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::tempdir;
use which::which;

pub fn require_executable(name: &str) {
    which(name).unwrap_or_else(|_| {
        panic!(
            "Executable {} not found. Please install it and make sure it is in your PATH.",
            name
        )
    });
}

fn create_latex_file(formula: &str, filename: &Path, use_dark_mode: bool) -> io::Result<()> {
    let mut file = File::create(filename)?;
    writeln!(file, "\\documentclass{{standalone}}")?;
    writeln!(file, "\\usepackage{{amsmath,amssymb,amsthm,xcolor}}")?;
    writeln!(file, "\\begin{{document}}")?;
    writeln!(file, "\\Large")?;
    if use_dark_mode {
        writeln!(file, "\\color{{white}}")?;
    }
    writeln!(file, "${}$", formula)?;
    writeln!(file, "\\end{{document}}")?;
    Ok(())
}

fn compile_latex_file(input_file: &Path, output_file: &Path) -> io::Result<()> {
    let mut cmd = Command::new("pdflatex");
    cmd.current_dir(input_file.parent().unwrap());
    cmd.arg(input_file);

    // TODO logging
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "pdflatex failed with code {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stdout)
            ),
        ));
    }

    // put at the correct place
    std::fs::rename(input_file.with_extension("pdf"), output_file)?;
    Ok(())
}

fn convert_pdf_to_svg(input_file: &Path, output_file: &Path) -> io::Result<()> {
    let mut cmd = Command::new("dvisvgm");
    cmd.arg("--pdf").arg(input_file).arg("-o").arg(output_file);

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
    let opt = resvg::usvg::Options {
        resources_dir: std::fs::canonicalize(input_file)
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf())),
        ..Default::default()
    };

    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();

    let svg_data = std::fs::read(input_file).unwrap();
    let mut tree = resvg::usvg::Tree::from_data(&svg_data, &opt).unwrap();
    tree.convert_text(&fontdb);

    let pixmap_size = tree.size.to_screen_size();
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).unwrap();
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

pub fn render_formula(formula: &str, out_path: &Path, use_dark_mode: bool) -> io::Result<PathBuf> {
    let tmp_dir = tempdir()?;
    let filename = format!("{}", uuid::Uuid::new_v4());
    let filename_tex = format!("{}.tex", filename);
    let filename_pdf = format!("{}.pdf", filename);
    let filename_svg = format!("{}.svg", filename);
    let filename_png = format!("{}.png", filename);
    let tmp_file = tmp_dir.path().join(filename_tex);
    let pdf_file = tmp_dir.path().join(filename_pdf);
    let svg_file = tmp_dir.path().join(filename_svg);
    // TODO Check if it is an directory
    let png_file = out_path.join(filename_png);

    create_latex_file(formula, &tmp_file, use_dark_mode)?;
    compile_latex_file(&tmp_file, &pdf_file)?;
    convert_pdf_to_svg(&pdf_file, &svg_file)?;
    convert_svg_to_png(&svg_file, &png_file)?;

    Ok(png_file)
}
