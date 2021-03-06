//! The webassembly client crate.

#![recursion_limit = "512"]
#![deny(
    anonymous_parameters,
    bare_trait_objects,
    clippy::clone_on_ref_ptr,
    clippy::float_cmp_const,
    clippy::if_not_else,
    clippy::unwrap_used
)]
#![cfg_attr(
    debug_assertions,
    allow(
        dead_code,
        unused_imports,
        unused_variables,
        clippy::match_single_binding,
    )
)]
#![cfg_attr(any(doc, not(debug_assertions)), warn(missing_docs))]
#![cfg_attr(
    not(debug_assertions),
    deny(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::dbg_macro,
        clippy::indexing_slicing,
    )
)]

use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod app;
pub mod camera;
pub mod config;
mod initial;
pub mod input;
pub mod render;
pub mod util;

/// Entry point.
#[wasm_bindgen(start)]
pub fn run_app() {
    std::panic::set_hook(Box::new(|info| {
        util::error_handler(&info.to_string());
    }));

    {
        let config = wasm_logger::Config::new(if cfg!(debug_assertions) {
            log::Level::Trace
        } else {
            log::Level::Info
        });
        wasm_logger::init(config);
    }

    App::<app::Mux>::new().mount_to_body();
}

/// Sets up legion ECS.
pub fn setup_ecs(setup: traffloat::SetupEcs) -> traffloat::SetupEcs {
    setup
        .uses(traffloat::setup_ecs)
        .uses(camera::setup_ecs)
        .uses(initial::setup_ecs)
        .uses(input::setup_ecs)
        .uses(render::setup_ecs)
}
