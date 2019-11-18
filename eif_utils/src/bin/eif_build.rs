// Copyright 2019 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#![deny(warnings)]
/// Simple utility tool for building an Eif file
///  cargo run -- --help  should be self explanatory.
/// Example of usage:
/// cargo run --target-dir=~/vmm-build -- --kernel bzImage \
///    --cmdline "reboot=k initrd=0x2000000,3228672 root=/dev/ram0 panic=1 pci=off nomodules \
///               console=ttyS0 i8042.noaux i8042.nomux i8042.nopnp i8042.dumbkbd"
///   --ramdisk  initramfs_x86.txt_part1.cpio.gz
///   --ramdisk  initramfs_x86.txt_part2.cpio.gz
///   --output   eif.bin
///
use std::path::Path;

use clap::{App, Arg};
use eif_utils::EifBuilder;
use sha2::{Digest, Sha256, Sha384, Sha512};
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::Write;

fn main() {
    let matches = App::new("Enclave image format builder")
        .about("Builds an eif file")
        .arg(
            Arg::with_name("kernel")
                .long("kernel")
                .value_name("FILE")
                .required(true)
                .help("Sets path to a bzImage file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("cmdline")
                .long("cmdline")
                .help("Sets the cmdline")
                .value_name("String")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .help("Specify output file path")
                .value_name("FILE")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ramdisk")
                .long("ramdisk")
                .value_name("FILE")
                .required(true)
                .help("Sets path to a ramdisk file representing a cpio.gz archive")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1),
        )
        .arg(
            Arg::with_name("sha256")
                .long("sha256")
                .help("Sets algorithm to be used for measuring the image")
                .group("measurement_alg"),
        )
        .arg(
            Arg::with_name("sha512")
                .long("sha512")
                .help("Sets algorithm to be used for measuring the image")
                .group("measurement_alg"),
        )
        .arg(
            Arg::with_name("sha384")
                .long("sha384")
                .help("Sets algorithm to be used for measuring the image")
                .group("measurement_alg"),
        )
        .get_matches();

    let kernel_path = matches
        .value_of("kernel")
        .expect("Kernel path is a mandatory option");

    let cmdline = matches
        .value_of("cmdline")
        .expect("Cmdline is a mandatory option");

    let sha512 = matches.is_present("sha512");
    let sha256 = matches.is_present("sha256");

    let ramdisks: Vec<&str> = matches
        .values_of("ramdisk")
        .expect("At least one ramdisk should be specified")
        .collect();

    let output_path = matches
        .value_of("output")
        .expect("Output file should be provided");

    if sha512 {
        build_eif(kernel_path, cmdline, ramdisks, output_path, Sha512::new());
    } else if sha256 {
        build_eif(kernel_path, cmdline, ramdisks, output_path, Sha256::new());
    } else {
        build_eif(kernel_path, cmdline, ramdisks, output_path, Sha384::new());
    }
}

pub fn build_eif<T: Digest + Debug + Write + Clone>(
    kernel_path: &str,
    cmdline: &str,
    ramdisks: Vec<&str>,
    output_path: &str,
    hasher: T,
) {
    let mut output_file = OpenOptions::new()
        .read(true)
        .create(true)
        .write(true)
        .truncate(true)
        .open(output_path)
        .expect("Could not create output file");

    let mut build = EifBuilder::new(&Path::new(kernel_path), cmdline.to_string(), hasher.clone());
    for ramdisk in ramdisks {
        build.add_ramdisk(Path::new(ramdisk));
    }

    build.write_to(&mut output_file);
    println!("Output written into {}", output_path);
    let measurements = build.boot_measurement();
    println!("BootMeasurement: {:?}: {:?}", hasher, measurements);
}