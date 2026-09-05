use bstr::{BString, ByteSlice};
use clap::{Parser, Subcommand};
use icy_sauce::{
    Capabilities, SauceDataType, SauceDate, SauceRecord, SauceRecordBuilder, StripMode, strip_sauce,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    comments: Option<Vec<String>>,
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
            comments: Some(
                sauce
                    .comments()
                    .iter()
                    .map(|c| c.to_str_lossy().into_owned())
                    .collect(),
            ),
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

    /// Resolve precedence before validation so overridden values are ignored.
    fn overlay(self, overrides: Self) -> Self {
        Self {
            title: overrides.title.or(self.title),
            author: overrides.author.or(self.author),
            group: overrides.group.or(self.group),
            date: overrides.date.or(self.date),
            comments: overrides.comments.or(self.comments),
            file_size: overrides.file_size.or(self.file_size),
            data_type: overrides.data_type.or(self.data_type),
            file_type: overrides.file_type.or(self.file_type),
            tinfo1: overrides.tinfo1.or(self.tinfo1),
            tinfo2: overrides.tinfo2.or(self.tinfo2),
            tinfo3: overrides.tinfo3.or(self.tinfo3),
            tinfo4: overrides.tinfo4.or(self.tinfo4),
            tflags: overrides.tflags.or(self.tflags),
            tinfos: overrides.tinfos.or(self.tinfos),
        }
    }

    /// Apply only supplied fields, preserving untouched bytes and raw fields.
    fn apply_to(
        self,
        mut builder: SauceRecordBuilder,
    ) -> Result<SauceRecordBuilder, Box<dyn std::error::Error>> {
        if let Some(value) = self.title {
            builder = builder.title(value.into())?;
        }
        if let Some(value) = self.author {
            builder = builder.author(value.into())?;
        }
        if let Some(value) = self.group {
            builder = builder.group(value.into())?;
        }
        if let Some(value) = self.date {
            builder = builder.date(parse_date(&value)?);
        }
        if let Some(values) = self.comments {
            builder = builder.clear_comments();
            for value in values {
                builder = builder.add_comment(value.into())?;
            }
        }
        if let Some(value) = self.file_size {
            builder = builder.file_size(value);
        }
        if let Some(value) = self.data_type {
            builder = builder.data_type(SauceDataType::from(value));
        }
        if let Some(value) = self.file_type {
            builder = builder.file_type(value);
        }
        if let Some(value) = self.tinfo1 {
            builder = builder.t_info1(value);
        }
        if let Some(value) = self.tinfo2 {
            builder = builder.t_info2(value);
        }
        if let Some(value) = self.tinfo3 {
            builder = builder.t_info3(value);
        }
        if let Some(value) = self.tinfo4 {
            builder = builder.t_info4(value);
        }
        if let Some(value) = self.tflags {
            builder = builder.t_flags(value);
        }
        if let Some(value) = self.tinfos {
            builder = builder.t_info_s(value.into())?;
        }
        Ok(builder)
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

        /// Title (max 35 bytes)
        #[arg(short, long)]
        title: Option<String>,

        /// Author (max 20 bytes)
        #[arg(short, long)]
        author: Option<String>,

        /// Group (max 20 bytes)
        #[arg(short, long)]
        group: Option<String>,

        /// Date in YYYYMMDD or YYYY-MM-DD format
        #[arg(short, long)]
        date: Option<String>,

        /// Comments (repeatable; replaces JSON comments when supplied)
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

        /// New title (max 35 bytes)
        #[arg(short, long)]
        title: Option<String>,

        /// New author (max 20 bytes)
        #[arg(short, long)]
        author: Option<String>,

        /// New group (max 20 bytes)
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
    let bytes = date_str.as_bytes();
    let mut compact = [0; 8];
    let digits = match bytes.len() {
        8 => bytes,
        10 if bytes[4] == b'-' && bytes[7] == b'-' => {
            compact[..4].copy_from_slice(&bytes[..4]);
            compact[4..6].copy_from_slice(&bytes[5..7]);
            compact[6..].copy_from_slice(&bytes[8..]);
            &compact
        }
        _ => return Err("Date must be in YYYYMMDD or YYYY-MM-DD format".into()),
    };
    SauceDate::from_bytes(digits)
        .ok_or_else(|| "Date must be in YYYYMMDD or YYYY-MM-DD format using ASCII digits".into())
}

/// Replace a file atomically, preserving its permissions and following symlinks.
/// A failed write leaves the original intact; the temporary file is cleaned up.
fn write_atomic(
    file: &Path,
    write: impl FnOnce(&mut fs::File) -> io::Result<()>,
) -> io::Result<()> {
    let path = fs::canonicalize(file)?;
    // Check write access without truncating; rename alone would also replace
    // read-only files when the containing directory is writable.
    let original = fs::OpenOptions::new().write(true).open(&path)?;
    let metadata = original.metadata()?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Not a regular file",
        ));
    }
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "File has no parent directory")
    })?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    write(temporary.as_file_mut())?;
    temporary
        .as_file()
        .set_permissions(metadata.permissions())?;
    temporary.as_file().sync_all()?;
    drop(original);
    temporary.persist(&path).map_err(|error| error.error)?;
    Ok(())
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

    // Zero denotes an unknown size for payloads too large for the wire field.
    let builder =
        SauceRecordBuilder::default().file_size(u32::try_from(content.len()).unwrap_or(0));
    let overrides = SauceJson {
        title,
        author,
        group,
        date,
        comments: (!comments.is_empty()).then_some(comments),
        data_type,
        file_type,
        tinfo1,
        tinfo2,
        tinfo3,
        tinfo4,
        tflags,
        tinfos,
        ..Default::default()
    };
    let builder = json
        .unwrap_or_default()
        .overlay(overrides)
        .apply_to(builder)?;

    let sauce = builder.build();

    let mut output = content;
    sauce.write(&mut output)?;
    write_atomic(file, |writer| writer.write_all(&output))?;

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

    // Strip the existing SAUCE to get the content
    let content = strip_sauce(&data, StripMode::LastStripFinalEof);

    // Preserve all untouched byte strings, file size, and raw format fields.
    let overrides = SauceJson {
        title,
        author,
        group,
        date,
        comments: if clear_comments {
            Some(Vec::new())
        } else {
            (!replace_comments.is_empty()).then_some(replace_comments)
        },
        data_type,
        file_type,
        tinfo1,
        tinfo2,
        tinfo3,
        tinfo4,
        tflags,
        tinfos,
        ..Default::default()
    };
    let mut builder = json
        .unwrap_or_default()
        .overlay(overrides)
        .apply_to(existing.to_builder())?;

    // Add new comments (always added on top of the above)
    for c in add_comments {
        builder = builder.add_comment(BString::from(c))?;
    }

    let sauce = builder.build();

    let mut output = content.to_vec();
    sauce.write(&mut output)?;
    write_atomic(file, |writer| writer.write_all(&output))?;

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
    write_atomic(file, |writer| writer.write_all(stripped))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_atomic_write_preserves_original_and_cleans_up() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("art.ans");
        fs::write(&file, b"original").unwrap();
        let result = write_atomic(&file, |writer| {
            writer.write_all(b"partial replacement")?;
            Err(io::Error::other("simulated write failure"))
        });
        assert!(result.is_err());
        assert_eq!(fs::read(&file).unwrap(), b"original");
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
}
