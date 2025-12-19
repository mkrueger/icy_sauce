use bstr::{BString, ByteSlice};
use clap::{Parser, Subcommand};
use icy_sauce::{
    Capabilities, SauceDataType, SauceDate, SauceRecord, SauceRecordBuilder, StripMode, strip_sauce,
};
use serde::{Deserialize, Serialize};
use std::{fs, io::Read, path::PathBuf, process::ExitCode};

/// JSON representation of SAUCE metadata
#[derive(Serialize, Deserialize, Default)]
struct SauceJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    comments: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_type: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tinfo1: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tinfo2: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tinfo3: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tinfo4: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tflags: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tinfos: Option<String>,
}

impl SauceJson {
    fn from_record(sauce: &SauceRecord) -> Self {
        let header = sauce.header();
        let date = sauce.date();
        let date_str = if date.year != 0 || date.month != 0 || date.day != 0 {
            Some(format!(
                "{:04}-{:02}-{:02}",
                date.year, date.month, date.day
            ))
        } else {
            None
        };

        SauceJson {
            title: Some(sauce.title().to_str_lossy().into_owned()),
            author: Some(sauce.author().to_str_lossy().into_owned()),
            group: Some(sauce.group().to_str_lossy().into_owned()),
            date: date_str,
            comments: sauce
                .comments()
                .iter()
                .map(|c| c.to_str_lossy().into_owned())
                .collect(),
            file_size: Some(sauce.file_size()),
            data_type: Some(u8::from(header.data_type)),
            file_type: Some(header.file_type),
            tinfo1: Some(header.t_info1),
            tinfo2: Some(header.t_info2),
            tinfo3: Some(header.t_info3),
            tinfo4: Some(header.t_info4),
            tflags: Some(header.t_flags),
            tinfos: if header.t_info_s.is_empty() {
                None
            } else {
                Some(header.t_info_s.to_str_lossy().into_owned())
            },
        }
    }
}

#[derive(Parser)]
#[command(name = "sauce")]
#[command(author = "Mike Krüger <mkrueger@posteo.de>")]
#[command(version)]
#[command(about = "View and modify SAUCE metadata in files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// View SAUCE metadata from a file
    View {
        /// File to read SAUCE from
        file: PathBuf,

        /// Show comments
        #[arg(short, long)]
        comments: bool,

        /// Show raw hex values
        #[arg(short, long)]
        raw: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Add SAUCE metadata to a file (fails if SAUCE already exists)
    Add {
        /// File to add SAUCE to
        file: PathBuf,

        /// Read SAUCE data from JSON (stdin if -, or file path)
        #[arg(long, value_name = "FILE")]
        from_json: Option<String>,

        /// Title (max 35 characters)
        #[arg(short, long)]
        title: Option<String>,

        /// Author (max 20 characters)
        #[arg(short, long)]
        author: Option<String>,

        /// Group (max 20 characters)
        #[arg(short, long)]
        group: Option<String>,

        /// Date in YYYYMMDD or YYYY-MM-DD format
        #[arg(short, long)]
        date: Option<String>,

        /// Comments (can be specified multiple times)
        #[arg(long)]
        comment: Vec<String>,

        /// Force overwrite if SAUCE already exists
        #[arg(short, long)]
        force: bool,

        // ─────────────────────────────────────────────────────────────────────
        // Raw SAUCE field options
        // ─────────────────────────────────────────────────────────────────────
        /// Set raw DataType (0=None, 1=Character, 2=Bitmap, 3=Vector, 4=Audio, 5=BinaryText, 6=XBin, 7=Archive, 8=Executable)
        #[arg(long, value_name = "0-8")]
        data_type: Option<u8>,

        /// Set raw FileType (0-255, meaning depends on DataType)
        #[arg(long, value_name = "0-255")]
        file_type: Option<u8>,

        /// Set raw TInfo1 field (16-bit, e.g., width/columns)
        #[arg(long, value_name = "0-65535")]
        tinfo1: Option<u16>,

        /// Set raw TInfo2 field (16-bit, e.g., height/lines)
        #[arg(long, value_name = "0-65535")]
        tinfo2: Option<u16>,

        /// Set raw TInfo3 field (16-bit, e.g., pixel depth)
        #[arg(long, value_name = "0-65535")]
        tinfo3: Option<u16>,

        /// Set raw TInfo4 field (16-bit, e.g., sample rate)
        #[arg(long, value_name = "0-65535")]
        tinfo4: Option<u16>,

        /// Set raw TFlags field (8-bit, ANSiFlags for Character types)
        #[arg(long, value_name = "0-255")]
        tflags: Option<u8>,

        /// Set raw TInfoS field (up to 22 bytes, e.g., font name)
        #[arg(long, value_name = "STRING")]
        tinfos: Option<String>,
    },

    /// Modify existing SAUCE metadata
    Alter {
        /// File to modify
        file: PathBuf,

        /// Read SAUCE data from JSON (stdin if -, or file path)
        #[arg(long, value_name = "FILE")]
        from_json: Option<String>,

        /// New title (max 35 characters)
        #[arg(short, long)]
        title: Option<String>,

        /// New author (max 20 characters)
        #[arg(short, long)]
        author: Option<String>,

        /// New group (max 20 characters)
        #[arg(short, long)]
        group: Option<String>,

        /// New date in YYYYMMDD or YYYY-MM-DD format
        #[arg(short, long)]
        date: Option<String>,

        /// Replace all comments with new ones
        #[arg(long)]
        comment: Vec<String>,

        /// Add a comment without replacing existing ones
        #[arg(long)]
        add_comment: Vec<String>,

        /// Clear all comments
        #[arg(long)]
        clear_comments: bool,

        // ─────────────────────────────────────────────────────────────────────
        // Raw SAUCE field options
        // ─────────────────────────────────────────────────────────────────────
        /// Set raw DataType (0=None, 1=Character, 2=Bitmap, 3=Vector, 4=Audio, 5=BinaryText, 6=XBin, 7=Archive, 8=Executable)
        #[arg(long, value_name = "0-8")]
        data_type: Option<u8>,

        /// Set raw FileType (0-255, meaning depends on DataType)
        #[arg(long, value_name = "0-255")]
        file_type: Option<u8>,

        /// Set raw TInfo1 field (16-bit, e.g., width/columns)
        #[arg(long, value_name = "0-65535")]
        tinfo1: Option<u16>,

        /// Set raw TInfo2 field (16-bit, e.g., height/lines)
        #[arg(long, value_name = "0-65535")]
        tinfo2: Option<u16>,

        /// Set raw TInfo3 field (16-bit, e.g., pixel depth)
        #[arg(long, value_name = "0-65535")]
        tinfo3: Option<u16>,

        /// Set raw TInfo4 field (16-bit, e.g., sample rate)
        #[arg(long, value_name = "0-65535")]
        tinfo4: Option<u16>,

        /// Set raw TFlags field (8-bit, ANSiFlags for Character types)
        #[arg(long, value_name = "0-255")]
        tflags: Option<u8>,

        /// Set raw TInfoS field (up to 22 bytes, e.g., font name)
        #[arg(long, value_name = "STRING")]
        tinfos: Option<String>,
    },

    /// Remove SAUCE metadata from a file
    Remove {
        /// File to remove SAUCE from
        file: PathBuf,

        /// Remove all SAUCE records (not just the last one)
        #[arg(short, long)]
        all: bool,

        /// Also remove EOF marker(s)
        #[arg(short, long)]
        strip_eof: bool,
    },

    /// Show information about the SAUCE format and field meanings
    Info,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::View {
            file,
            comments,
            raw,
            json,
        } => view_sauce(&file, comments, raw, json),
        Commands::Add {
            file,
            from_json,
            title,
            author,
            group,
            date,
            comment,
            force,
            data_type,
            file_type,
            tinfo1,
            tinfo2,
            tinfo3,
            tinfo4,
            tflags,
            tinfos,
        } => add_sauce(
            &file, from_json, title, author, group, date, comment, force, data_type, file_type,
            tinfo1, tinfo2, tinfo3, tinfo4, tflags, tinfos,
        ),
        Commands::Alter {
            file,
            from_json,
            title,
            author,
            group,
            date,
            comment,
            add_comment,
            clear_comments,
            data_type,
            file_type,
            tinfo1,
            tinfo2,
            tinfo3,
            tinfo4,
            tflags,
            tinfos,
        } => alter_sauce(
            &file,
            from_json,
            title,
            author,
            group,
            date,
            comment,
            add_comment,
            clear_comments,
            data_type,
            file_type,
            tinfo1,
            tinfo2,
            tinfo3,
            tinfo4,
            tflags,
            tinfos,
        ),
        Commands::Remove {
            file,
            all,
            strip_eof,
        } => remove_sauce(&file, all, strip_eof),
        Commands::Info => show_info(),
    }
}

fn view_sauce(
    file: &PathBuf,
    show_comments: bool,
    raw: bool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(file)?;

    let Some(sauce) = SauceRecord::from_bytes(&data)? else {
        if json {
            println!("null");
        } else {
            println!("No SAUCE record found in '{}'", file.display());
        }
        return Ok(());
    };

    if json {
        let json_data = SauceJson::from_record(&sauce);
        println!("{}", serde_json::to_string_pretty(&json_data)?);
    } else {
        println!("SAUCE Information for '{}'", file.display());
        println!("{}", "=".repeat(60));

        if raw {
            print_raw(&sauce);
        } else {
            print_formatted(&sauce);
        }

        let comments = sauce.comments();
        if show_comments && !comments.is_empty() {
            println!("\nComments ({}):", comments.len());
            println!("{}", "-".repeat(40));
            for (i, comment) in comments.iter().enumerate() {
                println!("  {}: {}", i + 1, comment.to_str_lossy());
            }
        }
    }

    Ok(())
}

fn print_formatted(sauce: &SauceRecord) {
    println!("Title:    {}", sauce.title().to_str_lossy());
    println!("Author:   {}", sauce.author().to_str_lossy());
    println!("Group:    {}", sauce.group().to_str_lossy());

    let date = sauce.date();
    if date.year != 0 || date.month != 0 || date.day != 0 {
        println!(
            "Date:     {:04}-{:02}-{:02}",
            date.year, date.month, date.day
        );
    } else {
        println!("Date:     (none)");
    }

    println!("FileSize: {}", sauce.file_size());

    if let Some(caps) = sauce.capabilities() {
        println!("\nCapabilities:");
        match caps {
            Capabilities::Character(c) => {
                println!("  Type:         Character");
                println!("  Format:       {:?}", c.format);
                println!("  Dimensions:   {}x{}", c.columns, c.lines);
                println!("  ICE Colors:   {}", c.ice_colors);
                println!("  Letter Spacing: {:?}", c.letter_spacing);
                println!("  Aspect Ratio: {:?}", c.aspect_ratio);
                if let Some(font) = c.font() {
                    println!("  Font:         {}", font.to_str_lossy());
                }
            }
            Capabilities::Binary(b) => {
                println!("  Type:         Binary");
                println!("  Format:       {:?}", b.format);
                println!("  Columns:      {}", b.columns);
                println!("  Lines:        {}", b.lines);
                println!("  ICE Colors:   {}", b.ice_colors);
                if let Some(font) = b.font() {
                    println!("  Font:         {}", font.to_str_lossy());
                }
            }
            Capabilities::Bitmap(b) => {
                println!("  Type:         Bitmap");
                println!("  Format:       {:?}", b.format);
                println!("  Dimensions:   {}x{}", b.width, b.height);
                println!("  Pixel Depth:  {}", b.pixel_depth);
            }
            Capabilities::Vector(v) => {
                println!("  Type:         Vector");
                println!("  Format:       {:?}", v.format);
            }
            Capabilities::Audio(a) => {
                println!("  Type:         Audio");
                println!("  Format:       {:?}", a.format);
                println!("  Sample Rate:  {}", a.sample_rate);
            }
            Capabilities::Archive(a) => {
                println!("  Type:         Archive");
                println!("  Format:       {:?}", a.format);
            }
            Capabilities::Executable(_) => {
                println!("  Type:         Executable");
            }
        }
    }
}

fn print_raw(sauce: &SauceRecord) {
    let header = sauce.header();
    println!("Title:         {:?}", sauce.title().as_slice());
    println!("Author:        {:?}", sauce.author().as_slice());
    println!("Group:         {:?}", sauce.group().as_slice());
    let date = sauce.date();
    println!(
        "Date:          {:04}{:02}{:02}",
        date.year, date.month, date.day
    );
    println!("FileSize:      {}", sauce.file_size());
    println!(
        "DataType:      {} ({:?})",
        u8::from(header.data_type),
        sauce.data_type()
    );
    println!("FileType:      {}", header.file_type);
    println!("TInfo1:        {}", header.t_info1);
    println!("TInfo2:        {}", header.t_info2);
    println!("TInfo3:        {}", header.t_info3);
    println!("TInfo4:        {}", header.t_info4);
    println!(
        "TFlags:        {} (0x{:02X})",
        header.t_flags, header.t_flags
    );
    println!("TInfoS:        {:?}", header.t_info_s.as_slice());
    println!("CommentCount:  {}", sauce.comments().len());
}

fn parse_date(date_str: &str) -> Result<SauceDate, Box<dyn std::error::Error>> {
    let cleaned = date_str.replace('-', "");
    if cleaned.len() != 8 {
        return Err("Date must be in YYYYMMDD or YYYY-MM-DD format".into());
    }

    let year: i32 = cleaned[0..4].parse()?;
    let month: u8 = cleaned[4..6].parse()?;
    let day: u8 = cleaned[6..8].parse()?;

    Ok(SauceDate::new(year, month, day))
}

/// Read JSON from a file path or stdin (if path is "-")
fn read_json_input(path: &str) -> Result<SauceJson, Box<dyn std::error::Error>> {
    let content = if path == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        fs::read_to_string(path)?
    };
    let json: SauceJson = serde_json::from_str(&content)?;
    Ok(json)
}

fn add_sauce(
    file: &PathBuf,
    from_json: Option<String>,
    title: Option<String>,
    author: Option<String>,
    group: Option<String>,
    date: Option<String>,
    comments: Vec<String>,
    force: bool,
    data_type: Option<u8>,
    file_type: Option<u8>,
    tinfo1: Option<u16>,
    tinfo2: Option<u16>,
    tinfo3: Option<u16>,
    tinfo4: Option<u16>,
    tflags: Option<u8>,
    tinfos: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load JSON input if provided
    let json = from_json.as_ref().map(|p| read_json_input(p)).transpose()?;

    let data = fs::read(file)?;

    // Check if SAUCE already exists
    if SauceRecord::from_bytes(&data)?.is_some() && !force {
        return Err("File already has SAUCE metadata. Use --force to overwrite.".into());
    }

    // Strip existing SAUCE if force is set
    let content = if force {
        strip_sauce(&data, StripMode::LastStripFinalEof).to_vec()
    } else {
        data
    };

    let mut builder = SauceRecordBuilder::default();

    // Apply JSON values first, then override with CLI arguments
    if let Some(ref j) = json {
        if let Some(ref t) = j.title {
            builder = builder.title(BString::from(t.as_str()))?;
        }
        if let Some(ref a) = j.author {
            builder = builder.author(BString::from(a.as_str()))?;
        }
        if let Some(ref g) = j.group {
            builder = builder.group(BString::from(g.as_str()))?;
        }
        if let Some(ref d) = j.date {
            builder = builder.date(parse_date(d)?);
        }
        for c in &j.comments {
            builder = builder.add_comment(BString::from(c.as_str()))?;
        }
        if let Some(dt) = j.data_type {
            builder = builder.data_type(SauceDataType::from(dt));
        }
        if let Some(ft) = j.file_type {
            builder = builder.file_type(ft);
        }
        if let Some(v) = j.tinfo1 {
            builder = builder.t_info1(v);
        }
        if let Some(v) = j.tinfo2 {
            builder = builder.t_info2(v);
        }
        if let Some(v) = j.tinfo3 {
            builder = builder.t_info3(v);
        }
        if let Some(v) = j.tinfo4 {
            builder = builder.t_info4(v);
        }
        if let Some(f) = j.tflags {
            builder = builder.t_flags(f);
        }
        if let Some(ref s) = j.tinfos {
            builder = builder.t_info_s(BString::from(s.as_str()))?;
        }
    }

    // CLI arguments override JSON values
    if let Some(t) = title {
        builder = builder.title(BString::from(t))?;
    }
    if let Some(a) = author {
        builder = builder.author(BString::from(a))?;
    }
    if let Some(g) = group {
        builder = builder.group(BString::from(g))?;
    }
    if let Some(d) = date {
        builder = builder.date(parse_date(&d)?);
    }
    // Only add CLI comments if no JSON was provided (to avoid duplicates)
    if json.is_none() {
        for c in comments {
            builder = builder.add_comment(BString::from(c))?;
        }
    }

    // Set raw fields if provided
    if let Some(dt) = data_type {
        builder = builder.data_type(SauceDataType::from(dt));
    }
    if let Some(ft) = file_type {
        builder = builder.file_type(ft);
    }
    if let Some(v) = tinfo1 {
        builder = builder.t_info1(v);
    }
    if let Some(v) = tinfo2 {
        builder = builder.t_info2(v);
    }
    if let Some(v) = tinfo3 {
        builder = builder.t_info3(v);
    }
    if let Some(v) = tinfo4 {
        builder = builder.t_info4(v);
    }
    if let Some(f) = tflags {
        builder = builder.t_flags(f);
    }
    if let Some(s) = tinfos {
        builder = builder.t_info_s(BString::from(s))?;
    }

    let sauce = builder.build();

    let mut output = content;
    sauce.write(&mut output)?;
    fs::write(file, output)?;

    println!("SAUCE metadata added to '{}'", file.display());
    Ok(())
}

fn alter_sauce(
    file: &PathBuf,
    from_json: Option<String>,
    title: Option<String>,
    author: Option<String>,
    group: Option<String>,
    date: Option<String>,
    replace_comments: Vec<String>,
    add_comments: Vec<String>,
    clear_comments: bool,
    data_type: Option<u8>,
    file_type: Option<u8>,
    tinfo1: Option<u16>,
    tinfo2: Option<u16>,
    tinfo3: Option<u16>,
    tinfo4: Option<u16>,
    tflags: Option<u8>,
    tinfos: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load JSON input if provided
    let json = from_json.as_ref().map(|p| read_json_input(p)).transpose()?;

    let data = fs::read(file)?;

    let Some(existing) = SauceRecord::from_bytes(&data)? else {
        return Err("No SAUCE record found. Use 'add' command to create one.".into());
    };

    // Check if any raw field is being set (from CLI or JSON)
    let using_raw_fields = data_type.is_some()
        || file_type.is_some()
        || tinfo1.is_some()
        || tinfo2.is_some()
        || tinfo3.is_some()
        || tinfo4.is_some()
        || tflags.is_some()
        || tinfos.is_some()
        || json.as_ref().map_or(false, |j| {
            j.data_type.is_some()
                || j.file_type.is_some()
                || j.tinfo1.is_some()
                || j.tinfo2.is_some()
                || j.tinfo3.is_some()
                || j.tinfo4.is_some()
                || j.tflags.is_some()
                || j.tinfos.is_some()
        });

    // Strip the existing SAUCE to get the content
    let content = strip_sauce(&data, StripMode::LastStripFinalEof);

    let mut builder = SauceRecordBuilder::default();

    // Determine final values: CLI overrides JSON, JSON overrides existing
    let final_title = title
        .or_else(|| json.as_ref().and_then(|j| j.title.clone()))
        .unwrap_or_else(|| existing.title().to_string());
    let final_author = author
        .or_else(|| json.as_ref().and_then(|j| j.author.clone()))
        .unwrap_or_else(|| existing.author().to_string());
    let final_group = group
        .or_else(|| json.as_ref().and_then(|j| j.group.clone()))
        .unwrap_or_else(|| existing.group().to_string());

    // Use new values or keep existing ones
    builder = builder.title(BString::from(final_title))?;
    builder = builder.author(BString::from(final_author))?;
    builder = builder.group(BString::from(final_group))?;

    // Handle date: CLI > JSON > existing
    if let Some(d) = date {
        builder = builder.date(parse_date(&d)?);
    } else if let Some(ref j) = json {
        if let Some(ref d) = j.date {
            builder = builder.date(parse_date(d)?);
        } else {
            let d = existing.date();
            if d.year != 0 || d.month != 0 || d.day != 0 {
                builder = builder.date(d);
            }
        }
    } else {
        let d = existing.date();
        if d.year != 0 || d.month != 0 || d.day != 0 {
            builder = builder.date(d);
        }
    }

    // Handle capabilities vs raw fields
    if using_raw_fields {
        // When using raw fields, set them directly instead of using capabilities
        // First copy existing raw values from the header
        let header = existing.header();

        // For each field: CLI > JSON > existing
        let final_data_type = data_type
            .or_else(|| json.as_ref().and_then(|j| j.data_type))
            .map(SauceDataType::from)
            .unwrap_or(header.data_type);
        let final_file_type = file_type
            .or_else(|| json.as_ref().and_then(|j| j.file_type))
            .unwrap_or(header.file_type);
        let final_tinfo1 = tinfo1
            .or_else(|| json.as_ref().and_then(|j| j.tinfo1))
            .unwrap_or(header.t_info1);
        let final_tinfo2 = tinfo2
            .or_else(|| json.as_ref().and_then(|j| j.tinfo2))
            .unwrap_or(header.t_info2);
        let final_tinfo3 = tinfo3
            .or_else(|| json.as_ref().and_then(|j| j.tinfo3))
            .unwrap_or(header.t_info3);
        let final_tinfo4 = tinfo4
            .or_else(|| json.as_ref().and_then(|j| j.tinfo4))
            .unwrap_or(header.t_info4);
        let final_tflags = tflags
            .or_else(|| json.as_ref().and_then(|j| j.tflags))
            .unwrap_or(header.t_flags);
        let final_tinfos = tinfos
            .or_else(|| json.as_ref().and_then(|j| j.tinfos.clone()))
            .map(BString::from)
            .unwrap_or_else(|| header.t_info_s.clone());

        builder = builder.data_type(final_data_type);
        builder = builder.file_type(final_file_type);
        builder = builder.t_info1(final_tinfo1);
        builder = builder.t_info2(final_tinfo2);
        builder = builder.t_info3(final_tinfo3);
        builder = builder.t_info4(final_tinfo4);
        builder = builder.t_flags(final_tflags);
        builder = builder.t_info_s(final_tinfos)?;
    } else {
        // Preserve capabilities when not using raw fields
        if let Some(caps) = existing.capabilities() {
            builder = builder.capabilities(caps)?;
        }
    }

    // Handle comments: clear_comments > replace_comments > json.comments > existing
    if clear_comments {
        // Don't add any comments
    } else if !replace_comments.is_empty() {
        for c in replace_comments {
            builder = builder.add_comment(BString::from(c))?;
        }
    } else if let Some(ref j) = json {
        // Use JSON comments if provided
        if !j.comments.is_empty() {
            for c in &j.comments {
                builder = builder.add_comment(BString::from(c.as_str()))?;
            }
        } else {
            // Keep existing comments
            for c in existing.comments() {
                builder = builder.add_comment(c.clone())?;
            }
        }
    } else {
        // Keep existing comments and optionally add new ones
        for c in existing.comments() {
            builder = builder.add_comment(c.clone())?;
        }
    }

    // Add new comments (always added on top of the above)
    for c in add_comments {
        builder = builder.add_comment(BString::from(c))?;
    }

    let sauce = builder.build();

    let mut output = content.to_vec();
    sauce.write(&mut output)?;
    fs::write(file, output)?;

    println!("SAUCE metadata updated in '{}'", file.display());
    Ok(())
}

fn remove_sauce(
    file: &PathBuf,
    all: bool,
    strip_eof: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(file)?;

    // Check if there's actually SAUCE to remove
    if SauceRecord::from_bytes(&data)?.is_none() {
        println!("No SAUCE record found in '{}'", file.display());
        return Ok(());
    }

    let mode = match (all, strip_eof) {
        (false, false) => StripMode::Last,
        (false, true) => StripMode::LastStripFinalEof,
        (true, false) => StripMode::All,
        (true, true) => StripMode::AllStripFinalEof,
    };

    let stripped = strip_sauce(&data, mode);
    fs::write(file, stripped)?;

    let records = if all {
        "All SAUCE records"
    } else {
        "SAUCE record"
    };
    println!("{} removed from '{}'", records, file.display());
    Ok(())
}

fn show_info() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        r#"
SAUCE - Standard Architecture for Universal Comment Extensions
===============================================================

SAUCE is a metadata format for files, commonly used in the ANSI art scene.
It stores information at the END of a file, making it non-intrusive.

SAUCE Header Structure (128 bytes):
------------------------------------
  ID        : "SAUCE" (5 bytes) - Magic identifier
  Version   : "00" (2 bytes) - Format version
  Title     : 35 bytes - File title
  Author    : 20 bytes - Creator name  
  Group     : 20 bytes - Group/organization
  Date      : 8 bytes - YYYYMMDD format
  FileSize  : 4 bytes - Original file size (little-endian)
  DataType  : 1 byte  - Content category (see below)
  FileType  : 1 byte  - Specific format within DataType
  TInfo1-4  : 2 bytes each - Type-specific info (e.g., dimensions)
  Comments  : 1 byte  - Number of comment lines (0-255)
  TFlags    : 1 byte  - Type-specific flags (ANSiFlags for text)
  TInfoS    : 22 bytes - String info (e.g., font name)

DataType Values:
-----------------
  0 = None        No specific type
  1 = Character   Text/ANSI art (ANS, ASC, PCB, etc.)
  2 = Bitmap      Graphics (GIF, JPG, PNG, BMP, etc.)
  3 = Vector      Vector graphics (DXF, etc.)
  4 = Audio       Sound files (MOD, S3M, XM, MP3, etc.)
  5 = BinaryText  Binary text with fixed 160-column width
  6 = XBin        Extended Binary format
  7 = Archive     Compressed archives (ZIP, ARJ, LZH, etc.)
  8 = Executable  Executable files

FileType (for DataType=1 Character):
-------------------------------------
  0 = ASCII       Plain ASCII text
  1 = ANSI        ANSI escape sequences
  2 = ANSiMation  Animated ANSI
  3 = RIP         Remote Imaging Protocol
  4 = PCBoard     PCBoard color codes
  5 = Avatar      Avatar codes
  6 = HTML        HTML markup
  7 = Source      Source code
  8 = TundraDraw  TundraDraw format

TInfo Fields (for Character types):
------------------------------------
  TInfo1 = Width in columns (or 0 for default 80)
  TInfo2 = Height in lines (0 = calculate from file)
  TFlags = ANSiFlags:
           Bit 0   : Non-blink mode (iCE colors)
           Bits 1-2: Letter spacing (0=legacy, 1=8px, 2=9px)
           Bits 3-4: Aspect ratio (0=legacy, 1=stretch, 2=square)
  TInfoS = Font name (e.g., "IBM VGA", "Topaz")

TInfo Fields (for Bitmap types):
---------------------------------
  TInfo1 = Width in pixels
  TInfo2 = Height in pixels  
  TInfo3 = Pixel depth (bits per pixel)

BinaryText Width Encoding (DataType=5):
----------------------------------------
  BinaryText files contain raw character+attribute pairs (2 bytes each).
  The width is encoded in the FileType field, NOT in TInfo1!
  
  FileType = (actual_width / 2) - 1
  
  This means:
    FileType 0   = 2 columns   (2 / 2 - 1 = 0)
    FileType 39  = 80 columns  (80 / 2 - 1 = 39)  <- most common
    FileType 79  = 160 columns (160 / 2 - 1 = 79)
    FileType 255 = 512 columns (maximum)
  
  To decode: actual_width = (FileType + 1) * 2
  
  Height is calculated from file size:
    height = file_size / (width * 2)
  
  TFlags works the same as for Character types (ANSiFlags).

Comments:
----------
  If Comments > 0, a comment block precedes the SAUCE record:
  - Starts with "COMNT" (5 bytes)
  - Followed by N x 64-byte comment lines
  - No null terminators within comments

File Layout:
-------------
  [Original File Content]
  [Optional: 0x1A EOF marker]
  [Optional: "COMNT" + comment lines]
  [SAUCE record - 128 bytes]

More info: https://www.acid.org/info/sauce/sauce.htm
"#
    );
    Ok(())
}
