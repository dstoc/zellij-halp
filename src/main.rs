use zellij_tile::{prelude::*, ZellijPlugin};

mod plugin;
mod renderer;

use crate::plugin::State;

register_plugin!(State);
