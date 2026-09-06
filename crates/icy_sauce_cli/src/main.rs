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
        // Exports are snapshots: represent empty values explicitly so importing
        // them clears these fields. Omitted input fields still mean no change.
        let date_str = Some(format!(
            "{:04}-{:02}-{:02}",
            date.year, date.month, date.day
        ));

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
            tinfos: Some(header.t_info_s.to_str_lossy().into_owned()),
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

        /// Remove all contiguous SAUCE records (not just the last one)
        ///
        /// If stripping stops with a SAUCE header remaining, valid trailing
        /// records are still removed, but the command reports partial removal
        /// and exits with status 1.
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
            eprintln!("Error: {}", display_text(e.to_string().as_bytes()));
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
            &file,
            from_json,
            SauceJson {
                title,
                author,
                group,
                date,
                comments: (!comment.is_empty()).then_some(comment),
                data_type,
                file_type,
                tinfo1,
                tinfo2,
                tinfo3,
                tinfo4,
                tflags,
                tinfos,
                ..Default::default()
            },
            force,
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
            SauceJson {
                title,
                author,
                group,
                date,
                comments: (!comment.is_empty()).then_some(comment),
                data_type,
                file_type,
                tinfo1,
                tinfo2,
                tinfo3,
                tinfo4,
                tflags,
                tinfos,
                ..Default::default()
            },
            add_comment,
            clear_comments,
        ),
        Commands::Remove {
            file,
            all,
            strip_eof,
        } => remove_sauce(&file, all, strip_eof),
        Commands::Info => show_info(),
    }
}

/// Render untrusted text without emitting terminal controls or bidi overrides.
fn display_text(bytes: &[u8]) -> String {
    let mut output = String::new();
    for c in bytes.to_str_lossy().chars() {
        if c.is_control()
            || matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        {
            output.extend(c.escape_default());
        } else {
            output.push(c);
        }
    }
    output
}

fn display_path(path: &Path) -> String {
    display_text(path.to_string_lossy().as_bytes())
}

/// Destructive commands must not continue when a comment block is ambiguous.
fn checked_strip(data: &[u8], mode: StripMode) -> Result<&[u8], Box<dyn std::error::Error>> {
    let stripped = strip_sauce(data, mode);
    if stripped.len() == data.len() {
        return Err("Cannot safely remove SAUCE: invalid or truncated comment block".into());
    }
    Ok(stripped)
}

fn view_sauce(
    file: &Path,
    show_comments: bool,
    raw: bool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(sauce) = SauceRecord::from_path(file)? else {
        if json {
            println!("null");
        } else {
            println!("No SAUCE record found in '{}'", display_path(file));
        }
        return Ok(());
    };

    if json {
        let json_data = SauceJson::from_record(&sauce);
        println!("{}", serde_json::to_string_pretty(&json_data)?);
    } else {
        println!("SAUCE Information for '{}'", display_path(file));
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
                println!("  {}: {}", i + 1, display_text(comment));
            }
        }
    }

    Ok(())
}

fn print_formatted(sauce: &SauceRecord) {
    println!("Title:    {}", display_text(sauce.title()));
    println!("Author:   {}", display_text(sauce.author()));
    println!("Group:    {}", display_text(sauce.group()));

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
                    println!("  Font:         {}", display_text(font));
                }
            }
            Capabilities::Binary(b) => {
                println!("  Type:         Binary");
                println!("  Format:       {:?}", b.format);
                println!("  Columns:      {}", b.columns);
                println!("  Lines:        {}", b.lines);
                println!("  ICE Colors:   {}", b.ice_colors);
                if let Some(font) = b.font() {
                    println!("  Font:         {}", display_text(font));
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

/// A pinned input target and the bytes read from that same file handle.
/// Resolve caller-supplied symlinks once, before reading, never again for writing.
struct FileSnapshot {
    path: PathBuf,
    identity: same_file::Handle,
    data: Vec<u8>,
}

impl FileSnapshot {
    fn read(file: &Path) -> io::Result<Self> {
        let path = fs::canonicalize(file)?;
        let mut original = fs::File::open(&path)?;
        if !original.metadata()?.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Not a regular file",
            ));
        }
        let identity = same_file::Handle::from_file(original.try_clone()?)?;
        let mut data = Vec::new();
        original.read_to_end(&mut data)?;
        Ok(Self {
            path,
            identity,
            data,
        })
    }

    /// Best-effort conflict detection, not a lock against uncooperative writers.
    fn verify_unchanged(&self) -> io::Result<()> {
        let mut current = fs::File::open(&self.path)?;
        let identity = same_file::Handle::from_file(current.try_clone()?)?;
        if identity != self.identity {
            return Err(io::Error::other(
                "File target changed since it was read; refusing replacement",
            ));
        }
        // Compare using bounded scratch space rather than another full-file copy.
        let mut buffer = [0; 8192];
        for expected in self.data.chunks(buffer.len()) {
            current.read_exact(&mut buffer[..expected.len()])?;
            if &buffer[..expected.len()] != expected {
                return Err(io::Error::other(
                    "File contents changed since they were read; refusing replacement",
                ));
            }
        }
        if current.read(&mut buffer[..1])? != 0 {
            return Err(io::Error::other(
                "File contents changed since they were read; refusing replacement",
            ));
        }
        Ok(())
    }
}

/// Atomically replace the pinned target, preserving permissions.
/// Failures before persistence leave the original intact and clean up the temporary file.
/// On Unix, sync the parent directory after persistence for rename durability.
/// A directory sync failure is reported after replacement; it cannot roll it back.
fn write_atomic(
    snapshot: &FileSnapshot,
    write: impl FnOnce(&mut fs::File) -> io::Result<()>,
) -> io::Result<()> {
    let path = &snapshot.path;
    snapshot.verify_unchanged()?;
    // Check write access without truncating; rename alone would also replace
    // read-only files when the containing directory is writable.
    let original = fs::OpenOptions::new().write(true).open(path)?;
    if same_file::Handle::from_file(original.try_clone()?)? != snapshot.identity {
        return Err(io::Error::other(
            "File target changed since it was read; refusing replacement",
        ));
    }
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
    // Open before replacement so failure to acquire the directory handle leaves
    // the original untouched, and retain it across the rename.
    #[cfg(unix)]
    let parent_directory = fs::File::open(parent)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    write(temporary.as_file_mut())?;
    temporary
        .as_file()
        .set_permissions(metadata.permissions())?;
    temporary.as_file().sync_all()?;
    snapshot.verify_unchanged()?;
    drop(original);
    temporary.persist(path).map_err(|error| error.error)?;
    #[cfg(unix)]
    parent_directory.sync_all().map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "File was replaced, but syncing the parent directory failed; crash durability is uncertain: {error}"
            ),
        )
    })?;
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
    file: &Path,
    from_json: Option<String>,
    overrides: SauceJson,
    force: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load JSON input if provided
    let json = from_json.as_ref().map(|p| read_json_input(p)).transpose()?;

    let snapshot = FileSnapshot::read(file)?;
    let data = &snapshot.data;

    // Check if SAUCE already exists
    let has_sauce = SauceRecord::from_bytes(data)?.is_some();
    if has_sauce && !force {
        return Err("File already has SAUCE metadata. Use --force to overwrite.".into());
    }

    // Strip existing SAUCE if force is set
    let content = if has_sauce {
        checked_strip(data, StripMode::LastStripFinalEof)?
    } else {
        data.as_slice()
    };

    // Zero denotes an unknown size for payloads too large for the wire field.
    let builder =
        SauceRecordBuilder::default().file_size(u32::try_from(content.len()).unwrap_or(0));
    let builder = json
        .unwrap_or_default()
        .overlay(overrides)
        .apply_to(builder)?;

    let sauce = builder.build();

    let mut output = content.to_vec();
    sauce.write(&mut output)?;
    write_atomic(&snapshot, |writer| writer.write_all(&output))?;

    println!("SAUCE metadata added to '{}'", display_path(file));
    Ok(())
}

fn alter_sauce(
    file: &Path,
    from_json: Option<String>,
    mut overrides: SauceJson,
    add_comments: Vec<String>,
    clear_comments: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load JSON input if provided
    let json = from_json.as_ref().map(|p| read_json_input(p)).transpose()?;

    let snapshot = FileSnapshot::read(file)?;
    let data = &snapshot.data;

    let Some(existing) = SauceRecord::from_bytes(data)? else {
        return Err("No SAUCE record found. Use 'add' command to create one.".into());
    };

    // Strip the existing SAUCE to get the content
    let content = checked_strip(data, StripMode::LastStripFinalEof)?;

    // Preserve all untouched byte strings, file size, and raw format fields.
    if clear_comments {
        overrides.comments = Some(Vec::new());
    }
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
    write_atomic(&snapshot, |writer| writer.write_all(&output))?;

    println!("SAUCE metadata updated in '{}'", display_path(file));
    Ok(())
}

fn remove_sauce(file: &Path, all: bool, strip_eof: bool) -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = FileSnapshot::read(file)?;
    let data = &snapshot.data;

    // Check if there's actually SAUCE to remove
    if SauceRecord::from_bytes(data)?.is_none() {
        println!("No SAUCE record found in '{}'", display_path(file));
        return Ok(());
    }

    let mode = match (all, strip_eof) {
        (false, false) => StripMode::Last,
        (false, true) => StripMode::LastStripFinalEof,
        (true, false) => StripMode::All,
        (true, true) => StripMode::AllStripFinalEof,
    };

    let stripped = checked_strip(data, mode)?;
    write_atomic(&snapshot, |writer| writer.write_all(stripped))?;

    // Stripping deliberately stops at malformed or unsupported headers. Report
    // the partial update rather than claiming that every record was removed.
    if all
        && !matches!(
            icy_sauce::header::SauceHeader::from_bytes(stripped),
            Ok(None)
        )
    {
        return Err(format!(
            "Partial removal in '{}': valid trailing records were removed, but a remaining SAUCE header was preserved because stripping stopped.",
            display_path(file)
        )
        .into());
    }

    let records = if all {
        "All contiguous SAUCE records"
    } else {
        "SAUCE record"
    };
    println!("{} removed from '{}'", records, display_path(file));
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
    4 = Audio       Sound files (MOD, S3M, XM, WAV, etc.)
    5 = BinaryText  Binary text with width encoded in FileType
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
  
    FileType = actual_width / 2
  
  This means:
    FileType 1   = 2 columns
    FileType 40  = 80 columns  <- most common
    FileType 80  = 160 columns
    FileType 255 = 510 columns (maximum)
    FileType 0 is invalid for BinaryText.
  
    To decode: actual_width = FileType * 2
  
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

    #[cfg(unix)]
    #[test]
    fn retargeted_input_symlink_never_overwrites_the_new_target() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().unwrap();
        let original = directory.path().join("original.ans");
        let other = directory.path().join("other.ans");
        let link = directory.path().join("link.ans");
        fs::write(&original, b"original").unwrap();
        fs::write(&other, b"unrelated").unwrap();
        symlink(&original, &link).unwrap();
        let snapshot = FileSnapshot::read(&link).unwrap();
        fs::remove_file(&link).unwrap();
        symlink(&other, &link).unwrap();

        write_atomic(&snapshot, |writer| writer.write_all(b"updated original")).unwrap();
        assert_eq!(fs::read(&original).unwrap(), b"updated original");
        assert_eq!(fs::read(&other).unwrap(), b"unrelated");
        assert_eq!(fs::read_link(&link).unwrap(), other);
    }

    #[test]
    fn replaced_target_is_rejected_even_with_identical_contents() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("art.ans");
        let old = directory.path().join("old.ans");
        fs::write(&file, b"original").unwrap();
        let snapshot = FileSnapshot::read(&file).unwrap();
        fs::rename(&file, &old).unwrap();
        fs::write(&file, b"original").unwrap();

        let error = write_atomic(&snapshot, |_| panic!("must reject before writing")).unwrap_err();
        assert!(error.to_string().contains("target changed"));
        assert_eq!(fs::read(&file).unwrap(), b"original");
        assert_eq!(fs::read(&old).unwrap(), b"original");
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    }

    #[test]
    fn concurrent_content_changes_are_preserved() {
        for contents in [b"modified".as_slice(), b"longer contents", b"short"] {
            for during_write in [false, true] {
                let directory = tempfile::tempdir().unwrap();
                let file = directory.path().join("art.ans");
                fs::write(&file, b"original").unwrap();
                let snapshot = FileSnapshot::read(&file).unwrap();
                if !during_write {
                    fs::write(&file, contents).unwrap();
                }
                let result = write_atomic(&snapshot, |writer| {
                    assert!(during_write, "must reject before writing");
                    writer.write_all(b"replacement")?;
                    fs::write(&file, contents)
                });
                assert!(result.is_err());
                assert_eq!(fs::read(&file).unwrap(), contents);
                assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
            }
        }
    }

    #[test]
    fn target_replaced_during_temporary_write_is_preserved() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("art.ans");
        let old = directory.path().join("old.ans");
        fs::write(&file, b"original").unwrap();
        let snapshot = FileSnapshot::read(&file).unwrap();
        let result = write_atomic(&snapshot, |writer| {
            writer.write_all(b"replacement")?;
            fs::rename(&file, &old)?;
            fs::write(&file, b"unrelated")
        });
        assert!(result.is_err());
        assert_eq!(fs::read(&file).unwrap(), b"unrelated");
        assert_eq!(fs::read(&old).unwrap(), b"original");
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    }

    #[test]
    fn failed_atomic_write_preserves_original_and_cleans_up() {
        let directory = tempfile::tempdir().unwrap();
        let file = directory.path().join("art.ans");
        fs::write(&file, b"original").unwrap();
        let snapshot = FileSnapshot::read(&file).unwrap();
        let result = write_atomic(&snapshot, |writer| {
            writer.write_all(b"partial replacement")?;
            Err(io::Error::other("simulated write failure"))
        });
        assert!(result.is_err());
        assert_eq!(fs::read(&file).unwrap(), b"original");
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }
}
