// Copyright IMD Technologies Ltd
use clap::Parser;
use fdt_edit::{Fdt, Node, Property};
use std::fs;
use std::process::ExitCode;

const CONFIGURATIONS: &str = "/configurations";

#[derive(Parser)]
#[command(
    name = "dtb",
    about = "Read or edit enabled device tree overlays within a qclinux_fit.image",
    author,
    after_help = "Run with --help to see examples",
    after_long_help = "To read the current FIT config tables:\n
    \t qcom-metadata-editor -d qclinux_fit.image \n\
    To edit config entry 1 to load qcs8550-imdt-sbc.dtb \n
    \t qcom-metadata-editor -d qclinux_fit.image -u -i 1 -f \"fdt-qcs8550-imdt-sbc.dtb\"",
    version
)]

struct Cli {
    /// Path to the qclinux_fit.img file to load
    qclinux_image: String,
    /// Update the config entry with the new .dtb/.dtbo list and comptible string
    #[arg(short, long)]
    update: bool,
    /// Config index (i.e 1,2,3,4 etc)
    #[arg(short, long)]
    index: Option<u8>,
    /// When --update is used, the compatible string for a config index will be updated to this value
    #[arg(short, long, required = false)]
    compatible: Option<String>,
    /// When --update is used, the list of device tree overlays to apply for an index wiil be updated to this value
    #[arg(short, long)]
    fdt: Option<Vec<String>>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<(), String> {
    println!("Parsing {}", cli.qclinux_image);

    let dtb = fs::read(&cli.qclinux_image)
        .map_err(|e| format!("Failed to read {}: {e}", cli.qclinux_image))?;

    let mut fdt = Fdt::from_bytes(&dtb)
        .map_err(|e| format!("Failed to parse {} as a DTB: {e:?}", cli.qclinux_image))?;

    if cli.update {
        return update(cli, &mut fdt);
    }

    match cli.index {
        Some(index) => {
            let path = config_path(index);
            let node = fdt
                .get_by_path(&path)
                .ok_or_else(|| format!("Failed to find config node {path}"))?;
            print_config(node.as_node());
        }
        None => {
            let configs = fdt.get_by_path(CONFIGURATIONS).ok_or_else(|| {
                format!(
                    "Failed to find {CONFIGURATIONS} node in {}",
                    cli.qclinux_image
                )
            })?;
            for id in configs.as_node().children() {
                if let Some(child) = fdt.node(*id) {
                    print_config(child);
                }
            }
        }
    }

    Ok(())
}

/// Apply the requested edits to `fdt`, then write the whole tree back over the file
/// it came from.
fn update(cli: &Cli, fdt: &mut Fdt) -> Result<(), String> {
    let index = cli
        .index
        .ok_or("--update needs a config index, pass --index")?;
    if cli.compatible.is_none() && cli.fdt.is_none() {
        return Err("--update needs something to change, pass --compatible and/or --fdt".into());
    }

    let path = config_path(index);
    let id = fdt
        .get_by_path_id(&path)
        .ok_or_else(|| format!("Failed to find config node {path}"))?;
    let node = fdt.node_mut(id).expect("the id came from this tree");

    println!("Updating {path}");
    if let Some(compatible) = &cli.compatible {
        let mut prop = Property::new("compatible", Vec::new());
        prop.set_string(compatible);
        node.set_property(prop);
        println!("\t\t compatible: {compatible}");
    }
    if let Some(overlays) = &cli.fdt {
        let overlays: Vec<&str> = overlays.iter().map(String::as_str).collect();
        let mut prop = Property::new("fdt", Vec::new());
        prop.set_string_ls(&overlays);
        node.set_property(prop);
        println!("\t\t fdt: {}", overlays.join(" "));
    }

    let out = fdt.encode();
    fs::write(&cli.qclinux_image, out.as_ref())
        .map_err(|e| format!("Failed to write {}: {e}", cli.qclinux_image))?;
    println!("Wrote {}", cli.qclinux_image);

    Ok(())
}

fn config_path(index: u8) -> String {
    format!("{CONFIGURATIONS}/conf-{index}")
}

fn print_config(node: &Node) {
    println!("\t {}", node.name());
    for name in ["compatible", "fdt"] {
        match node.get_property(name) {
            Some(prop) => {
                let values: Vec<&str> = prop.as_str_iter().collect();
                println!("\t\t {name}: {}", values.join(" "));
            }
            None => println!("\t\t {name}: <none>"),
        }
    }
}
