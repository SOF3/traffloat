//! Crate to generate docs.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use structopt::StructOpt;

mod assets;
mod buildings;
mod cargo;
mod gas;
mod liquid;
mod manifest;
mod opts;
mod reactions;

fn main() -> Result<()> {
    let opts = opts::Opts::from_args();
    fs::create_dir_all(&opts.root_dir).context("Could not create --root-dir")?;
    let root_dir = opts
        .root_dir
        .canonicalize()
        .context("--root-dir could not be canonicalized")?;
    let mut assets = assets::Pool::new(root_dir.join("docs"), String::from("assets"))?;

    let relativize = |path: &Path| {
        let path = path.canonicalize().context("Could not canonicalize path")?;
        let stripped = path
            .strip_prefix(&root_dir.join("docs"))
            .context("Canonicalized path is not under root_dir")?;
        Ok(stripped.to_path_buf())
    };

    let buildings_index = buildings::gen_buildings(&opts, &mut assets, relativize)
        .context("Generating buildings guide")?;
    let reactions_index = reactions::gen_reactions(&opts, &mut assets, relativize)
        .context("Generating reactions guide")?;
    let cargos_index =
        cargo::gen_cargos(&opts, &mut assets, relativize).context("Generating cargos guide")?;
    let gases_index =
        gas::gen_gases(&opts, &mut assets, relativize).context("Generating gases guide")?;
    let liquids_index =
        liquid::gen_liquids(&opts, &mut assets, relativize).context("Generating liquids guide")?;

    let index = vec![
        manifest::Nav::Index {
            title: String::from("Buildings"),
            items: buildings_index,
        },
        manifest::Nav::Index {
            title: String::from("Mechanisms"),
            items: reactions_index,
        },
        manifest::Nav::Index {
            title: String::from("Cargo"),
            items: cargos_index,
        },
        manifest::Nav::Index {
            title: String::from("Gases"),
            items: gases_index,
        },
        manifest::Nav::Index {
            title: String::from("Liquids"),
            items: liquids_index,
        },
    ];

    let mkdocs_yml = opts.root_dir.join("mkdocs.yml");
    let mkdocs_yml =
        fs::File::create(mkdocs_yml).context("Could not open mkdocs.yml for writing")?;
    let favicon_path = opts.client_dir.join("static/favicon.ico");
    let favicon_path = assets
        .map(&favicon_path)
        .context("Resolving favicon path")?;
    let manifest = manifest::Mkdocs {
        site_name: "Traffloat user guide",
        site_url: opts.site_url.clone().unwrap_or_else(String::new),
        use_directory_urls: opts.site_url.is_some(),
        site_author: "SOFe",
        repo_url: "https://github.com/traffloat/traffloat",
        repo_name: "traffloat/traffloat",
        copyright: "Licensed under AGPL 3.0",
        theme: manifest::Theme {
            name: "material",
            favicon: favicon_path.clone(),
            logo: favicon_path,
            features: &[],
        },
        markdown_extensions: &["attr_list"],
        nav: index,
    };
    serde_yaml::to_writer(mkdocs_yml, &manifest).context("YAML formatting error")?;

    Ok(())
}