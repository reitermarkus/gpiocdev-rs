// SPDX-FileCopyrightText: 2023 Kent Gibson <warthog618@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

use anyhow::anyhow;
use clap::Parser;
use super::common::{self, emit_error};
use gpiocdev::AbiVersion;
use std::fs;

#[derive(Debug, Parser)]
#[command(aliases(["p"]))]
pub struct Opts {
    #[command(flatten)]
    emit: common::EmitOpts,
}

pub fn cmd(opts: &Opts) -> bool {
    if let Ok(v) = fs::read_to_string("/proc/version") {
        let mut f = v.split_ascii_whitespace();
        if let Some(v) = f.nth(2) {
            println!("Kernel {}", v);
            if opts.emit.verbose {
                print_unsupported_features(v)
            }
        }
    } else {
        println!("Kernel unknown");
    }
    print_abi_support(&opts.emit)
}

fn print_abi_support(opts: &common::EmitOpts) -> bool {
    let versions = [AbiVersion::V1, AbiVersion::V2];
    let mut success = false;
    for v in versions {
        match gpiocdev::supports_abi_version(v) {
            Err(gpiocdev::Error::NoGpioChips()) => {
                println!("No available gpiochips");
                return false;
            }
            Ok(()) => {
                println!("{} is supported.", v);
                success = true;
            }
            Err(e) => emit_error(opts, &anyhow!(e)),
        }
    }
    success
}

fn print_unsupported_features(version: &str) {
    if let Some((major, minor)) = parse_version(version) {
        if major >= 6 || (major == 5 && minor >= 19) {
            println!("Kernel supports all uAPI features.");
        } else if major == 5 && minor > 10 && minor < 19 {
            println!("Kernel does not support event_clock hte (added in 5.19).");
        } else if major == 5 && minor == 10 {
            println!("Kernel does not support event_clock realtime (added in 5.11) or hte (added in 5.19).");
        } else if major < 5 || (major == 5 && minor < 5) {
            println!("Kernel does not support bias or reconfigure (added in 5.5).");
        }
    }
}

fn parse_version(version: &str) -> Option<(u32, u32)> {
    let mut f = version.split('.');
    let major = f.next()?.parse().ok()?;
    let minor = f.next()?.parse().ok()?;
    Some((major, minor))
}
