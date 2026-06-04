use std::{
    env,
    error::Error,
    ffi::OsStr,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::Duration,
};

const LIMINE_VERSION: &str = "v12.2.0";
const LIMINE_URL: &str =
    "https://github.com/limine-bootloader/limine/releases/download/v12.2.0/limine-binary.zip";
const IMAGE_SIZE: u64 = 64 * 1024 * 1024;
const SECTOR_SIZE: usize = 512;
const PARTITION_LBA: u32 = 2048;
const LIMINE_DIR: &str = ".tools/limine/v12.2.0";
const IMAGE_PATH: &str = "target/oxid.img";
const KERNEL_PATH: &str = "target/x86_64-unknown-none/debug/oxid";
const SERIAL_LOG_PATH: &str = "target/serial.log";
const SMOKE_BOOT_MARKER: &str = "Oxid kernel initialized";
const SMOKE_EXCEPTION_MARKER: &str = "Exception: Breakpoint";

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return Err(
            "usage: cargo run --manifest-path tools/xtask/Cargo.toml -- <image|run-qemu>".into(),
        );
    };

    match command.as_str() {
        "check" => check(),
        "image" => image(),
        "run-qemu" => run_qemu(),
        "smoke-exception" => smoke_exception(),
        "smoke-qemu" => smoke_qemu(),
        _ => Err(format!("unknown xtask command: {command}").into()),
    }
}

fn check() -> Result<(), Box<dyn Error>> {
    run("cargo", ["test"])?;
    run(
        "cargo",
        ["test", "--manifest-path", "tools/xtask/Cargo.toml"],
    )?;
    run("cargo", ["kbuild"])?;
    run("cargo", ["ktest-build"])?;
    image()
}

fn image() -> Result<(), Box<dyn Error>> {
    image_with_kernel_features(&[])
}

fn image_with_kernel_features(features: &[&str]) -> Result<(), Box<dyn Error>> {
    ensure_limine()?;
    build_kernel(features)?;

    let limine_dir = Path::new(LIMINE_DIR);
    let limine_bios_sys = limine_dir.join("limine-bios.sys");
    let limine_tool = limine_executable(limine_dir);
    let kernel = Path::new(KERNEL_PATH);
    let config = Path::new("boot/limine.conf");
    let image = Path::new(IMAGE_PATH);

    let files = [
        ImageFile::new("/limine-bios.sys", fs::read(&limine_bios_sys)?),
        ImageFile::new("/limine.conf", fs::read(config)?),
        ImageFile::new("/boot/limine/limine-bios.sys", fs::read(&limine_bios_sys)?),
        ImageFile::new("/boot/limine/limine.conf", fs::read(config)?),
        ImageFile::new("/boot/oxid.elf", fs::read(kernel)?),
    ];

    if let Some(parent) = image.parent() {
        fs::create_dir_all(parent)?;
    }

    Fat16Image::create(image, IMAGE_SIZE, &files)?;
    run_path(&limine_tool, ["bios-install", IMAGE_PATH])?;

    println!("created {}", image.display());
    Ok(())
}

fn build_kernel(features: &[&str]) -> Result<(), Box<dyn Error>> {
    let mut command = Command::new("cargo");
    command.args([
        "build",
        "-Zbuild-std=core,compiler_builtins",
        "-Zbuild-std-features=compiler-builtins-mem",
        "-Zjson-target-spec",
        "--target",
        "x86_64-unknown-none.json",
    ]);

    if !features.is_empty() {
        command.arg("--features").arg(features.join(","));
    }

    run_command(&mut command)
}

fn run_qemu() -> Result<(), Box<dyn Error>> {
    image()?;
    run(
        "qemu-system-x86_64",
        [
            "-m",
            "256M",
            "-drive",
            "format=raw,file=target/oxid.img",
            "-boot",
            "c",
            "-serial",
            "stdio",
            "-monitor",
            "none",
            "-no-reboot",
            "-no-shutdown",
        ],
    )
}

fn smoke_qemu() -> Result<(), Box<dyn Error>> {
    image_with_kernel_features(&[])?;

    run_qemu_smoke(SMOKE_BOOT_MARKER)
}

fn smoke_exception() -> Result<(), Box<dyn Error>> {
    image_with_kernel_features(&["exception-smoke"])?;

    run_qemu_smoke(SMOKE_EXCEPTION_MARKER)
}

fn run_qemu_smoke(marker: &str) -> Result<(), Box<dyn Error>> {
    let serial_log = Path::new(SERIAL_LOG_PATH);
    if serial_log.exists() {
        fs::remove_file(serial_log)?;
    }

    let mut child = Command::new("qemu-system-x86_64")
        .args([
            "-m",
            "256M",
            "-drive",
            "format=raw,file=target/oxid.img,if=ide,index=0,media=disk",
            "-boot",
            "c",
            "-serial",
            "file:target/serial.log",
            "-display",
            "none",
            "-monitor",
            "none",
            "-no-reboot",
            "-no-shutdown",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    thread::sleep(Duration::from_secs(5));

    if child.try_wait()?.is_none() {
        child.kill()?;
    }
    let _ = child.wait();

    let serial = fs::read_to_string(serial_log)?;
    if !serial.contains(marker) {
        return Err(format!(
            "QEMU smoke test did not find `{marker}` in {SERIAL_LOG_PATH}\n{serial}"
        )
        .into());
    }

    println!("{serial}");
    Ok(())
}

fn ensure_limine() -> Result<(), Box<dyn Error>> {
    let limine_dir = Path::new(LIMINE_DIR);
    if limine_dir.join("limine-bios.sys").exists() && limine_executable(limine_dir).exists() {
        return Ok(());
    }

    fs::create_dir_all(limine_dir)?;

    let archive = Path::new(".tools/limine/limine-binary.zip");
    if let Some(parent) = archive.parent() {
        fs::create_dir_all(parent)?;
    }

    download_file(LIMINE_URL, archive)?;
    extract_limine_files(archive, limine_dir)?;

    println!(
        "Limine {LIMINE_VERSION} is ready at {}",
        limine_dir.display()
    );
    Ok(())
}

fn limine_executable(limine_dir: &Path) -> PathBuf {
    if cfg!(windows) {
        limine_dir.join("limine.exe")
    } else {
        limine_dir.join("limine")
    }
}

fn download_file(url: &str, output: &Path) -> Result<(), Box<dyn Error>> {
    println!("Downloading {url}");

    let status = Command::new("curl")
        .args(["-L", "--fail", "--output"])
        .arg(output)
        .arg(url)
        .status();

    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("curl failed with status {status}").into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Err("curl is required to download Limine automatically".into())
        }
        Err(error) => Err(error.into()),
    }
}

fn extract_limine_files(archive: &Path, limine_dir: &Path) -> Result<(), Box<dyn Error>> {
    if cfg!(windows) {
        extract_limine_files_with_powershell(archive, limine_dir)
    } else {
        extract_limine_files_with_unzip(archive, limine_dir)
    }
}

fn extract_limine_files_with_powershell(
    _archive: &Path,
    limine_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let extract_dir = Path::new(".tools/limine/extract");
    if extract_dir.exists() {
        fs::remove_dir_all(extract_dir)?;
    }
    fs::create_dir_all(extract_dir)?;

    run_path(
        Path::new("powershell"),
        [
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            "Expand-Archive -Force -Path '.tools/limine/limine-binary.zip' -DestinationPath '.tools/limine/extract'",
        ],
    )?;

    copy_limine_files_from_tree(extract_dir, limine_dir)?;
    fs::remove_dir_all(extract_dir)?;
    Ok(())
}

fn extract_limine_files_with_unzip(
    archive: &Path,
    limine_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let list = Command::new("unzip").arg("-Z1").arg(archive).output();
    let output = match list {
        Ok(output) if output.status.success() => output,
        Ok(_) => return Err("unzip failed to list Limine archive".into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(
                "unzip is required to extract Limine automatically on this platform".into(),
            );
        }
        Err(error) => return Err(error.into()),
    };

    let listing = String::from_utf8(output.stdout)?;
    for wanted in ["limine", "limine-bios.sys"] {
        let Some(entry) = listing.lines().find(|line| line.ends_with(wanted)) else {
            return Err(format!("could not find {wanted} in Limine archive").into());
        };

        let data = unzip_file_to_memory(archive, entry)?;
        let destination = limine_dir.join(wanted);
        fs::write(&destination, data)?;

        #[cfg(unix)]
        if wanted == "limine" {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&destination)?.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&destination, permissions)?;
        }
    }

    Ok(())
}

fn unzip_file_to_memory(archive: &Path, entry: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut child = Command::new("unzip")
        .arg("-p")
        .arg(archive)
        .arg(entry)
        .stdout(Stdio::piped())
        .spawn()?;

    let mut data = Vec::new();
    child.stdout.as_mut().unwrap().read_to_end(&mut data)?;
    let status = child.wait()?;
    if !status.success() {
        return Err(format!("failed to extract {entry}").into());
    }

    Ok(data)
}

fn copy_limine_files_from_tree(source: &Path, target: &Path) -> Result<(), Box<dyn Error>> {
    for wanted in [limine_executable_name(), "limine-bios.sys"] {
        let Some(found) = find_file(source, wanted)? else {
            return Err(format!("could not find {wanted} in Limine archive").into());
        };
        fs::copy(found, target.join(wanted))?;
    }
    Ok(())
}

fn limine_executable_name() -> &'static str {
    if cfg!(windows) {
        "limine.exe"
    } else {
        "limine"
    }
}

fn find_file(root: &Path, name: &str) -> Result<Option<PathBuf>, Box<dyn Error>> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file(&path, name)? {
                return Ok(Some(found));
            }
        } else if path.file_name().and_then(OsStr::to_str) == Some(name) {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

fn run<I, S>(program: &str, args: I) -> Result<(), Box<dyn Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_path(Path::new(program), args)
}

fn run_path<I, S>(program: &Path, args: I) -> Result<(), Box<dyn Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let status = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .status()?;

    check_status(program, status)
}

fn run_command(command: &mut Command) -> Result<(), Box<dyn Error>> {
    let program = command.get_program().to_owned();
    let status = command.stdin(Stdio::null()).status()?;

    check_status(Path::new(&program), status)
}

fn check_status(program: &Path, status: std::process::ExitStatus) -> Result<(), Box<dyn Error>> {
    if !status.success() {
        return Err(format!("command failed: {}", program.display()).into());
    }

    Ok(())
}

struct ImageFile {
    path: &'static str,
    data: Vec<u8>,
}

impl ImageFile {
    fn new(path: &'static str, data: Vec<u8>) -> Self {
        Self { path, data }
    }
}

struct Fat16Image {
    bytes: Vec<u8>,
    partition_start: usize,
    fat_start: usize,
    sectors_per_fat: usize,
    root_dir_start: usize,
    root_dir_sectors: usize,
    data_start: usize,
    sectors_per_cluster: usize,
    next_cluster: u16,
}

impl Fat16Image {
    fn create(path: &Path, size: u64, files: &[ImageFile]) -> Result<(), Box<dyn Error>> {
        let mut image = Self::new(size as usize)?;
        image.write_mbr();
        image.write_boot_sector();
        image.init_fats();
        image.write_files(files)?;
        fs::write(path, image.bytes)?;
        Ok(())
    }

    fn new(size: usize) -> Result<Self, Box<dyn Error>> {
        if size % SECTOR_SIZE != 0 {
            return Err("image size must be sector-aligned".into());
        }

        let partition_start = PARTITION_LBA as usize;
        let partition_sectors = size / SECTOR_SIZE - partition_start;
        let sectors_per_cluster = 4;
        let root_dir_entries: usize = 512;
        let root_dir_sectors = (root_dir_entries * 32).div_ceil(SECTOR_SIZE);
        let reserved = 1;
        let fats = 2;
        let mut sectors_per_fat = 1;

        loop {
            let data_sectors =
                partition_sectors - reserved - fats * sectors_per_fat - root_dir_sectors;
            let clusters = data_sectors / sectors_per_cluster;
            let required = ((clusters + 2) * 2).div_ceil(SECTOR_SIZE);
            if required == sectors_per_fat {
                break;
            }
            sectors_per_fat = required;
        }

        let fat_start = partition_start + reserved;
        let root_dir_start = fat_start + fats * sectors_per_fat;
        let data_start = root_dir_start + root_dir_sectors;

        Ok(Self {
            bytes: vec![0; size],
            partition_start,
            fat_start,
            sectors_per_fat,
            root_dir_start,
            root_dir_sectors,
            data_start,
            sectors_per_cluster,
            next_cluster: 2,
        })
    }

    fn write_mbr(&mut self) {
        let partition_sectors = (self.bytes.len() / SECTOR_SIZE - self.partition_start) as u32;
        let entry = 446;
        self.bytes[entry] = 0x80;
        self.bytes[entry + 1] = 0xff;
        self.bytes[entry + 2] = 0xff;
        self.bytes[entry + 3] = 0xff;
        self.bytes[entry + 4] = 0x0e;
        self.bytes[entry + 5] = 0xff;
        self.bytes[entry + 6] = 0xff;
        self.bytes[entry + 7] = 0xff;
        self.write_u32(entry + 8, PARTITION_LBA);
        self.write_u32(entry + 12, partition_sectors);
        self.bytes[510] = 0x55;
        self.bytes[511] = 0xaa;
    }

    fn write_boot_sector(&mut self) {
        let offset = self.partition_start * SECTOR_SIZE;
        let total_sectors = (self.bytes.len() / SECTOR_SIZE - self.partition_start) as u32;

        self.bytes[offset..offset + 3].copy_from_slice(&[0xeb, 0x3c, 0x90]);
        self.bytes[offset + 3..offset + 11].copy_from_slice(b"OXIDFAT ");
        self.write_u16(offset + 11, SECTOR_SIZE as u16);
        self.bytes[offset + 13] = self.sectors_per_cluster as u8;
        self.write_u16(offset + 14, 1);
        self.bytes[offset + 16] = 2;
        self.write_u16(offset + 17, 512);
        self.write_u16(offset + 19, 0);
        self.bytes[offset + 21] = 0xf8;
        self.write_u16(offset + 22, self.sectors_per_fat as u16);
        self.write_u16(offset + 24, 63);
        self.write_u16(offset + 26, 255);
        self.write_u32(offset + 28, PARTITION_LBA);
        self.write_u32(offset + 32, total_sectors);
        self.bytes[offset + 36] = 0x80;
        self.bytes[offset + 38] = 0x29;
        self.write_u32(offset + 39, 0x0d10_0001);
        self.bytes[offset + 43..offset + 54].copy_from_slice(b"OXID       ");
        self.bytes[offset + 54..offset + 62].copy_from_slice(b"FAT16   ");
        self.bytes[offset + 510] = 0x55;
        self.bytes[offset + 511] = 0xaa;
    }

    fn init_fats(&mut self) {
        self.set_fat_entry(0, 0xfff8);
        self.set_fat_entry(1, 0xffff);
    }

    fn write_files(&mut self, files: &[ImageFile]) -> Result<(), Box<dyn Error>> {
        let boot_cluster = self.allocate_directory_cluster(0)?;
        let limine_cluster = self.allocate_directory_cluster(boot_cluster)?;
        let mut root_entries = Vec::new();
        root_entries.push(DirEntry::directory("BOOT", boot_cluster));

        let mut boot_entries = vec![
            DirEntry::dot(boot_cluster),
            DirEntry::dotdot(0),
            DirEntry::directory("LIMINE", limine_cluster),
        ];

        let mut limine_entries = vec![
            DirEntry::dot(limine_cluster),
            DirEntry::dotdot(boot_cluster),
        ];

        for file in files {
            let (directory, name) = split_path(file.path)?;
            let cluster = self.write_file_data(&file.data)?;
            let size = file.data.len() as u32;
            match directory {
                "/" => root_entries.extend(DirEntry::file(name, cluster, size)),
                "/boot" => boot_entries.extend(DirEntry::file(name, cluster, size)),
                "/boot/limine" => limine_entries.extend(DirEntry::file(name, cluster, size)),
                _ => return Err(format!("unsupported image path: {}", file.path).into()),
            }
        }

        self.write_root_dir(&root_entries)?;
        self.write_cluster_dir(boot_cluster, &boot_entries)?;
        self.write_cluster_dir(limine_cluster, &limine_entries)?;
        Ok(())
    }

    fn allocate_directory_cluster(&mut self, parent: u16) -> Result<u16, Box<dyn Error>> {
        let cluster = self.allocate_clusters(1)?[0];
        if parent == 0 {
            self.set_fat_entry(cluster, 0xffff);
        }
        Ok(cluster)
    }

    fn write_file_data(&mut self, data: &[u8]) -> Result<u16, Box<dyn Error>> {
        let bytes_per_cluster = self.sectors_per_cluster * SECTOR_SIZE;
        let cluster_count = data.len().max(1).div_ceil(bytes_per_cluster);
        let clusters = self.allocate_clusters(cluster_count)?;

        for pair in clusters.windows(2) {
            self.set_fat_entry(pair[0], pair[1]);
        }
        self.set_fat_entry(*clusters.last().unwrap(), 0xffff);

        for (index, chunk) in data.chunks(bytes_per_cluster).enumerate() {
            let offset = self.cluster_offset(clusters[index]);
            self.bytes[offset..offset + chunk.len()].copy_from_slice(chunk);
        }

        Ok(clusters[0])
    }

    fn allocate_clusters(&mut self, count: usize) -> Result<Vec<u16>, Box<dyn Error>> {
        let mut clusters = Vec::with_capacity(count);
        for _ in 0..count {
            if self.next_cluster == u16::MAX {
                return Err("FAT image ran out of clusters".into());
            }
            clusters.push(self.next_cluster);
            self.next_cluster += 1;
        }
        Ok(clusters)
    }

    fn write_root_dir(&mut self, entries: &[DirEntry]) -> Result<(), Box<dyn Error>> {
        let start = self.root_dir_start * SECTOR_SIZE;
        let capacity = self.root_dir_sectors * SECTOR_SIZE;
        write_dir_entries(&mut self.bytes[start..start + capacity], entries)
    }

    fn write_cluster_dir(
        &mut self,
        cluster: u16,
        entries: &[DirEntry],
    ) -> Result<(), Box<dyn Error>> {
        let start = self.cluster_offset(cluster);
        let capacity = self.sectors_per_cluster * SECTOR_SIZE;
        write_dir_entries(&mut self.bytes[start..start + capacity], entries)
    }

    fn set_fat_entry(&mut self, cluster: u16, value: u16) {
        for fat in 0..2 {
            let offset =
                (self.fat_start + fat * self.sectors_per_fat) * SECTOR_SIZE + cluster as usize * 2;
            self.write_u16(offset, value);
        }
    }

    fn cluster_offset(&self, cluster: u16) -> usize {
        let sector = self.data_start + (cluster as usize - 2) * self.sectors_per_cluster;
        sector * SECTOR_SIZE
    }

    fn write_u16(&mut self, offset: usize, value: u16) {
        self.bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn write_u32(&mut self, offset: usize, value: u32) {
        self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

#[derive(Clone)]
struct DirEntry([u8; 32]);

impl DirEntry {
    fn directory(name: &str, cluster: u16) -> Self {
        let mut entry = Self::short(name, "", 0x10, cluster, 0);
        entry.0[11] = 0x10;
        entry
    }

    fn dot(cluster: u16) -> Self {
        Self::short(".", "", 0x10, cluster, 0)
    }

    fn dotdot(cluster: u16) -> Self {
        Self::short("..", "", 0x10, cluster, 0)
    }

    fn file(name: &str, cluster: u16, size: u32) -> Vec<Self> {
        let (short_name, needs_lfn) = short_name(name);
        let mut entries = Vec::new();
        if needs_lfn {
            entries.extend(long_name_entries(name, &short_name));
        }
        entries.push(Self::raw_short(short_name, 0x20, cluster, size));
        entries
    }

    fn short(name: &str, ext: &str, attributes: u8, cluster: u16, size: u32) -> Self {
        let mut short = [b' '; 11];
        for (index, byte) in name.bytes().take(8).enumerate() {
            short[index] = byte.to_ascii_uppercase();
        }
        for (index, byte) in ext.bytes().take(3).enumerate() {
            short[8 + index] = byte.to_ascii_uppercase();
        }
        Self::raw_short(short, attributes, cluster, size)
    }

    fn raw_short(name: [u8; 11], attributes: u8, cluster: u16, size: u32) -> Self {
        let mut entry = [0; 32];
        entry[0..11].copy_from_slice(&name);
        entry[11] = attributes;
        entry[26..28].copy_from_slice(&cluster.to_le_bytes());
        entry[28..32].copy_from_slice(&size.to_le_bytes());
        Self(entry)
    }
}

fn write_dir_entries(target: &mut [u8], entries: &[DirEntry]) -> Result<(), Box<dyn Error>> {
    let needed = entries.len() * 32;
    if needed + 32 > target.len() {
        return Err("directory is full".into());
    }

    for (index, entry) in entries.iter().enumerate() {
        target[index * 32..index * 32 + 32].copy_from_slice(&entry.0);
    }
    target[needed] = 0;
    Ok(())
}

fn split_path(path: &str) -> Result<(&str, &str), Box<dyn Error>> {
    let Some(index) = path.rfind('/') else {
        return Err(format!("invalid image path: {path}").into());
    };
    let directory = if index == 0 { "/" } else { &path[..index] };
    let name = &path[index + 1..];
    Ok((directory, name))
}

fn short_name(name: &str) -> ([u8; 11], bool) {
    let mut parts = name.splitn(2, '.');
    let base = parts.next().unwrap_or("");
    let ext = parts.next().unwrap_or("");
    let valid = base.len() <= 8
        && ext.len() <= 3
        && !base.is_empty()
        && base.bytes().chain(ext.bytes()).all(is_short_name_byte);

    if valid {
        let mut short = [b' '; 11];
        for (index, byte) in base.bytes().enumerate() {
            short[index] = byte.to_ascii_uppercase();
        }
        for (index, byte) in ext.bytes().enumerate() {
            short[8 + index] = byte.to_ascii_uppercase();
        }
        return (short, false);
    }

    let mut compact = Vec::new();
    for byte in base.bytes().filter(u8::is_ascii_alphanumeric).take(6) {
        compact.push(byte.to_ascii_uppercase());
    }
    while compact.len() < 6 {
        compact.push(b'_');
    }

    let mut short = [b' '; 11];
    short[..6].copy_from_slice(&compact[..6]);
    short[6] = b'~';
    short[7] = b'1';
    for (index, byte) in ext
        .bytes()
        .filter(u8::is_ascii_alphanumeric)
        .take(3)
        .enumerate()
    {
        short[8 + index] = byte.to_ascii_uppercase();
    }
    (short, true)
}

fn is_short_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'~' | b'-')
}

fn long_name_entries(name: &str, short_name: &[u8; 11]) -> Vec<DirEntry> {
    let mut utf16: Vec<u16> = name.encode_utf16().collect();
    utf16.push(0);
    while !utf16.len().is_multiple_of(13) {
        utf16.push(0xffff);
    }

    let checksum = short_name_checksum(short_name);
    let chunks: Vec<&[u16]> = utf16.chunks(13).collect();
    let mut entries = Vec::new();

    for (index, chunk) in chunks.iter().enumerate().rev() {
        let ordinal = (index + 1) as u8 | if index == chunks.len() - 1 { 0x40 } else { 0 };
        let mut entry = [0xff; 32];
        entry[0] = ordinal;
        entry[11] = 0x0f;
        entry[13] = checksum;
        write_lfn_chars(&mut entry, chunk);
        entries.push(DirEntry(entry));
    }

    entries
}

fn write_lfn_chars(entry: &mut [u8; 32], chars: &[u16]) {
    let slots = [1usize, 3, 5, 7, 9, 14, 16, 18, 20, 22, 24, 28, 30];
    for (slot, value) in slots.iter().zip(chars.iter().copied()) {
        entry[*slot..*slot + 2].copy_from_slice(&value.to_le_bytes());
    }
}

fn short_name_checksum(short_name: &[u8; 11]) -> u8 {
    let mut sum = 0u8;
    for byte in short_name {
        sum = ((sum & 1) << 7).wrapping_add(sum >> 1).wrapping_add(*byte);
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::{limine_executable_name, short_name, split_path};

    #[test]
    fn splits_image_paths() {
        assert_eq!(split_path("/boot/oxid.elf").unwrap(), ("/boot", "oxid.elf"));
        assert_eq!(split_path("/limine.conf").unwrap(), ("/", "limine.conf"));
    }

    #[test]
    fn keeps_valid_short_names() {
        let (_, needs_lfn) = short_name("oxid.elf");

        assert!(!needs_lfn);
    }

    #[test]
    fn flags_limine_names_as_long_names() {
        let (_, needs_lfn) = short_name("limine-bios.sys");

        assert!(needs_lfn);
    }

    #[test]
    fn uses_platform_limine_executable_name() {
        if cfg!(windows) {
            assert_eq!(limine_executable_name(), "limine.exe");
        } else {
            assert_eq!(limine_executable_name(), "limine");
        }
    }
}
