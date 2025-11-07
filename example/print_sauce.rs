use clap::Parser;
use icy_sauce::SauceInformation;
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
    let sauce = match SauceInformation::read(&data) {
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

    match sauce.date() {
        Ok(date) => {
            println!("Date:     {}", date);
        }
        Err(err) => {
            println!("Date:     {}", err);
        }
    }

    println!("Type:     {:?}", sauce.data_type());

    if sauce.file_size() > 0 {
        println!("FileSize: {} bytes", sauce.file_size());
    }

    // Print format-specific information
    println!();
    match sauce.data_type() {
        icy_sauce::SauceDataType::Character => {
            if let Ok(caps) = sauce.get_character_capabilities() {
                println!("Character File Information:");
                println!("  Format:     {:?}", caps.format);
                println!(
                    "  Dimensions: {}x{} (columns x rows)",
                    caps.width, caps.height
                );
                if caps.use_ice {
                    println!("  iCE Colors: Yes");
                }
                println!("  Letter Spacing: {:?}", caps.letter_spacing);
                println!("  Aspect Ratio:   {:?}", caps.aspect_ratio);
                if let Some(font) = &caps.font_opt() {
                    if !font.is_empty() {
                        println!("  Font:       {}", font);
                    }
                }
            }
        }
        icy_sauce::SauceDataType::BinaryText | icy_sauce::SauceDataType::XBin => {
            if let Ok(caps) = sauce.get_bin_capabilities() {
                println!("Binary Text Information:");
                println!("  Format:     {:?}", caps.format);
                println!("  Width:      {} columns", caps.width);
                if caps.format == icy_sauce::bin_caps::BinFormat::XBin {
                    println!("  Height:     {} rows", caps.height);
                } else if let Some(height) = caps.calculate_binary_text_height(sauce.file_size()) {
                    println!("  Height:     {} rows (calculated)", height);
                }
                if let Some(font) = &caps.font_name {
                    if !font.is_empty() {
                        println!("  Font:       {}", font);
                    }
                }
            }
        }
        icy_sauce::SauceDataType::Bitmap | icy_sauce::SauceDataType::Vector => {
            if let Ok(caps) = sauce.get_pixel_capabilities() {
                println!("Graphics File Information:");
                println!("  Format:     {:?}", caps.format);
                println!("  Dimensions: {}x{} pixels", caps.width, caps.height);
                println!("  Color Depth: {} bits", caps.pixel_depth);
            }
        }
        icy_sauce::SauceDataType::Audio => {
            if let Ok(caps) = sauce.get_audio_capabilities() {
                println!("Audio File Information:");
                println!("  Format:     {:?}", caps.format);
                if caps.sample_rate > 0 {
                    println!("  Sample Rate: {} Hz", caps.sample_rate);
                }
            }
        }
        icy_sauce::SauceDataType::Archive => {
            if let Ok(caps) = sauce.get_archive_capabilities() {
                println!("Archive File Information:");
                println!("  Format:     {:?}", caps.format);
            }
        }
        icy_sauce::SauceDataType::Executable => {
            if let Ok(_) = sauce.get_executable_capabilities() {
                println!("Executable File Information:");
                println!(" -");
            }
        }
        _ => {}
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
        println!("File Type:      {}", sauce.header().data_type);
        println!("TInfo1:         {}", sauce.header().t_info1);
        println!("TInfo2:         {}", sauce.header().t_info2);
        println!("TInfo3:         {}", sauce.header().t_info3);
        println!("TInfo4:         {}", sauce.header().t_info4);
        println!(
            "TFlags:         0b{:08b} (0x{:02X})",
            sauce.header().t_flags,
            sauce.header().t_flags
        );
        if !sauce.header().t_info_s.is_empty() {
            println!("TInfoS:         {:?}", sauce.header().t_info_s);
        }
        println!("Record Size:    {} bytes", sauce.info_len());
    }
}
