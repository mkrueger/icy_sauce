use clap::Parser;
use icy_sauce::{Capabilities, SauceRecord};
use std::fs;
use std::path::PathBuf;
use std::process;

/// Print SAUCE metadata from files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File to read SAUCE information from
    #[arg(value_name = "FILE")]
    file: PathBuf,

    /// Show comments if present
    #[arg(short, long)]
    comments: bool,

    /// Show raw technical details
    #[arg(short = 'r', long)]
    raw: bool,
}

fn main() {
    let args = Args::parse();

    // Read the file
    let data = match fs::read(&args.file) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", args.file.display(), err);
            process::exit(1);
        }
    };

    // Parse SAUCE information
    let sauce = match SauceRecord::from_bytes(&data) {
        Ok(Some(sauce)) => sauce,
        Ok(None) => {
            println!("No SAUCE record found in '{}'", args.file.display());
            process::exit(0);
        }
        Err(err) => {
            eprintln!("Error parsing SAUCE: {}", err);
            process::exit(1);
        }
    };

    // Print basic information
    println!("SAUCE Information for '{}'", args.file.display());
    println!("{}", "=".repeat(60));

    if !sauce.title().is_empty() {
        println!("Title:    {}", sauce.title());
    }
    if !sauce.author().is_empty() {
        println!("Author:   {}", sauce.author());
    }
    if !sauce.group().is_empty() {
        println!("Group:    {}", sauce.group());
    }

    // Handle date which returns Result
    println!("Date:     {}", sauce.date());

    println!("Type:     {}", sauce.data_type());

    if sauce.file_size() > 0 {
        println!("FileSize: {} bytes", sauce.file_size());
    }

    // Print format-specific information using unified capabilities
    if let Some(caps) = sauce.capabilities() {
        println!();
        match caps {
            Capabilities::Character(caps) => {
                println!("Character File Information:");
                println!("  Format:     {:?}", caps.format);
                println!(
                    "  Dimensions: {}x{} (columns x rows)",
                    caps.columns, caps.lines
                );
                if caps.ice_colors {
                    println!("  iCE Colors: Yes");
                }
                println!("  Letter Spacing: {:?}", caps.letter_spacing);
                println!("  Aspect Ratio:   {:?}", caps.aspect_ratio);
                // Use font() method instead of font field
                if let Some(font) = caps.font() {
                    if !font.is_empty() {
                        println!("  Font:       {}", font);
                    }
                }
            }
            Capabilities::Binary(caps) => {
                println!("Binary Text Information:");
                println!("  Format:     {:?}", caps.format);
                println!("  Width:      {} columns", caps.columns);
                if caps.format == icy_sauce::binary::BinaryFormat::XBin {
                    println!("  Height:     {} rows", caps.lines);
                } else if let Some(height) =
                    caps.binary_text_height_from_file_size(sauce.file_size())
                {
                    println!("  Height:     {} rows (calculated)", height);
                }
            }
            Capabilities::Vector(caps) => {
                println!("Vector Graphics File Information:");
                println!("  Format:     {:?}", caps.format);
            }
            Capabilities::Bitmap(caps) => {
                println!("Bitmap File Information:");
                println!("  Format:     {:?}", caps.format);
                println!("  Dimensions: {}x{} pixels", caps.width, caps.height);
                println!("  Color Depth: {} bits", caps.pixel_depth);
            }
            Capabilities::Audio(caps) => {
                println!("Audio File Information:");
                println!("  Format:     {:?}", caps.format);
                if caps.sample_rate > 0 {
                    println!("  Sample Rate: {} Hz", caps.sample_rate);
                }
            }
            Capabilities::Archive(caps) => {
                println!("Archive File Information:");
                println!("  Format:     {:?}", caps.format);
            }
            Capabilities::Executable(_caps) => {
                println!("Executable File Information");
                // ExecutableCaps doesn't have a format field
            }
        }
    }

    // Print comments if requested
    if args.comments && sauce.comments().len() > 0 {
        println!();
        println!("Comments ({}):", sauce.comments().len());
        println!("{}", "-".repeat(40));
        for (i, comment) in sauce.comments().iter().enumerate() {
            println!("{:3}: {}", i + 1, comment);
        }
    }

    // Print raw technical details if requested
    if args.raw {
        println!();
        println!("Raw SAUCE Data:");
        println!("{}", "-".repeat(40));

        // Access raw header fields through the header
        let header = sauce.header();
        println!("File Type:      {}", header.file_type);
        println!("TInfo1:         {}", header.t_info1);
        println!("TInfo2:         {}", header.t_info2);
        println!("TInfo3:         {}", header.t_info3);
        println!("TInfo4:         {}", header.t_info4);
        println!(
            "TFlags:         0b{:08b} (0x{:02X})",
            header.t_flags, header.t_flags
        );
        if !header.t_info_s.is_empty() {
            println!("TInfoS:         {:?}", header.t_info_s);
        }
        println!("Record Size:    {} bytes", sauce.record_len());
    }
}
