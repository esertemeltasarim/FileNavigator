use rug::{Float, Integer};
use sha2::{Sha256, Digest};
use std::{
    fs::{self, File},
    io::{self, Read, Write, BufReader, BufWriter, Cursor},
    path::{Path, PathBuf},
};

// 1M-bit precision
const PRECISION: u32 = 1_000_000;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SizeUnit {
    KB,
    MB,
    GB,
}

pub struct TranscendentalEngine {
    digits: Vec<u8>,
    pub chunk_size: usize,
}

impl TranscendentalEngine {
    pub fn new(seed: &str, chunk_size: u64, unit: SizeUnit) -> Self {
        let multiplier = match unit {
            SizeUnit::KB => 1024,
            SizeUnit::MB => 1024 * 1024,
            SizeUnit::GB => 1024 * 1024 * 1024,
        };
        let bytes = chunk_size.saturating_mul(multiplier) as usize;

        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        let hash = hasher.finalize();

        // r in (0.09, 0.99)
        let r = {
            let int = Integer::from_digits(&hash[0..16], rug::integer::Order::Lsf);
            let f = int.to_f64().unwrap();
            Float::with_val(PRECISION, (f % 0.9) + 0.09)
        };

        let omega = {
            let int = Integer::from_digits(&hash[16..32], rug::integer::Order::Lsf);
            let f = int.to_f64().unwrap();
            Float::with_val(PRECISION, f % 1e6)
        };

        let phi = {
            let int = Integer::from_digits(&hash[0..16], rug::integer::Order::Lsf);
            let f = int.to_f64().unwrap();
            Float::with_val(PRECISION, f % std::f64::consts::TAU)
        };

        // Generate chaotic sequence using a transcendental equation
        let mut digits = Vec::with_capacity(bytes);
        let mut x = Float::with_val(PRECISION, 0);
        let c = &Float::with_val(PRECISION, 1) - &r;

        for n in 0.. {
            // term = c * r^n * sin(n*omega + phi)
            let mut term = Float::with_val(PRECISION, 0);
            let rn = r.clone().pow(n as u32);
            let angle = Float::with_val(PRECISION, &omega * n + &phi);
            term.assign(&c * rn * angle.sin());
            x += &term;

            let digit = (x.to_integer().unwrap().to_u8().unwrap() % 10) as u8;
            digits.push(digit);

            if digits.len() >= bytes {
                break;
            }
        }

        TranscendentalEngine {
            digits,
            chunk_size: bytes,
        }
    }

    pub fn process<R, W>(&self, mut reader: R, mut writer: W, on_progress: impl Fn(u64)) -> io::Result<()>
    where
        R: Read,
        W: Write,
    {
        let mut buffer = vec![0u8; 4096];
        let mut total_read: u64 = 0;

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            let processed: Vec<u8> = buffer[..bytes_read]
                .iter()
                .enumerate()
                .map(|(i, b)| {
                    let k = &self.digits[i % self.digits.len()];
                    b.wrapping_add(*k)
                })
                .collect();

            writer.write_all(&processed)?;
            total_read += bytes_read as u64;
            on_progress(total_read);
        }

        Ok(())
    }
}

pub fn detect_parts(input_path: &Path) -> io::Result<Vec<PathBuf>> {
    let stem = input_path.file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file stem"))?;
    
    let dir = input_path.parent().unwrap_or(Path::new("."));
    
    let mut parts: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with(&format!("{}", stem)) && n.contains(".part"))
            .unwrap_or(false)
        )
        .collect();
    
    parts.sort();
    Ok(parts)
}

pub fn encrypt_file(
    input_path: &Path,
    output_path: &Path,
    seed: &str,
    chunk_size: u64,
    unit: SizeUnit,
    progress_callback: impl Fn(u64, u64),
) -> io::Result<()> {
    let meta = fs::metadata(input_path)?;
    let total_size = meta.len();
    let engine = TranscendentalEngine::new(seed, chunk_size, unit);

    let infile = File::open(input_path)?;
    let outfile = File::create(output_path)?;

    let progress_fn = |bytes_processed: u64| {
        progress_callback(bytes_processed, total_size);
    };

    engine.process(BufReader::new(infile), BufWriter::new(outfile), progress_fn)?;
    
    Ok(())
}

pub fn decrypt_file(
    input_path: &Path,
    output_path: &Path,
    seed: &str,
    chunk_size: u64,
    unit: SizeUnit,
    progress_callback: impl Fn(u64, u64),
) -> io::Result<()> {
    let meta = fs::metadata(input_path)?;
    let total_size = meta.len();
    let engine = TranscendentalEngine::new(seed, chunk_size, unit);

    let infile = File::open(input_path)?;
    let outfile = File::create(output_path)?;

    let progress_fn = |bytes_processed: u64| {
        progress_callback(bytes_processed, total_size);
    };

    engine.process(BufReader::new(infile), BufWriter::new(outfile), progress_fn)?;
    
    Ok(())
}

pub fn split_and_encrypt(
    input_path: &Path,
    output_dir: &Path,
    seed: &str,
    chunk_size: u64,
    unit: SizeUnit,
    progress_callback: impl Fn(u64, u64),
) -> io::Result<Vec<PathBuf>> {
    let meta = fs::metadata(input_path)?;
    let total_size = meta.len();
    let engine = TranscendentalEngine::new(seed, chunk_size, unit);
    let mut part_number = 0;
    let mut reader = BufReader::new(File::open(input_path)?);
    let mut output_parts = Vec::new();
    let mut total_processed: u64 = 0;

    loop {
        let mut buffer = vec![0u8; engine.chunk_size];
        let bytes_read = reader.read(&mut buffer)?;
        
        if bytes_read == 0 {
            break;
        }
        
        let part_name = format!(
            "{}{}.part{}",
            input_path.file_stem().unwrap().to_string_lossy(),
            "",
            part_number
        );
        
        let output_path = output_dir.join(part_name);
        output_parts.push(output_path.clone());
        
        let writer = BufWriter::new(File::create(&output_path)?);
        
        let progress_fn = |bytes_processed: u64| {
            let current_progress = total_processed + bytes_processed;
            progress_callback(current_progress, total_size);
        };

        engine.process(Cursor::new(&buffer[..bytes_read]), writer, progress_fn)?;
        
        total_processed += bytes_read as u64;
        part_number += 1;
    }

    Ok(output_parts)
}

pub fn decrypt_parts(
    parts: &[PathBuf],
    output_path: &Path,
    seed: &str,
    progress_callback: impl Fn(u64, u64),
) -> io::Result<()> {
    let mut total_size: u64 = 0;
    for part in parts {
        total_size += fs::metadata(part)?.len();
    }
    
    let mut outfile = BufWriter::new(File::create(output_path)?);
    let mut total_processed: u64 = 0;
    
    for part in parts {
        let engine = TranscendentalEngine::new(seed, 0, SizeUnit::MB);
        let infile = File::open(part)?;
        let part_size = fs::metadata(part)?.len();
        
        let progress_fn = |bytes_processed: u64| {
            let current_progress = total_processed + bytes_processed;
            progress_callback(current_progress, total_size);
        };
        
        engine.process(BufReader::new(infile), Cursor::new(Vec::new()), |_| {})?;
        
        total_processed += part_size;
    }
    
    Ok(())
}
