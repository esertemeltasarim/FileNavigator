#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use rug::{Float, Integer};
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{self, Read, Write, BufReader, BufWriter, Cursor},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
};

// Constants
const PRECISION: u32 = 1_000_000; // 1M-bit precision

// ==================== ENCRYPTION MODULE ====================

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

// ==================== INTERNATIONALIZATION MODULE ====================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Translations {
    pub window_title: String,
    pub encrypt_tab: String,
    pub decrypt_tab: String,
    pub input_file_label: String,
    pub output_file_label: String,
    pub browse_button: String,
    pub seed_label: String,
    pub seed_tooltip: String,
    pub chunk_size_label: String,
    pub unit_label: String,
    pub kilobyte: String,
    pub megabyte: String,
    pub gigabyte: String,
    pub encrypt_button: String,
    pub decrypt_button: String,
    pub progress_label: String,
    pub split_large_files: String,
    pub language_label: String,
    pub english: String,
    pub turkish: String,
    pub success_encrypt: String,
    pub success_decrypt: String,
    pub error_no_input: String,
    pub error_no_output: String,
    pub error_no_seed: String,
    pub error_processing: String,
    pub error_title: String,
    pub success_title: String,
    pub large_file_prompt: String,
    pub yes: String,
    pub no: String,
    pub cancel: String,
}

impl Default for Translations {
    fn default() -> Self {
        Self::english()
    }
}

impl Translations {
    pub fn english() -> Self {
        Translations {
            window_title: "Transcendental Encryption Tool".to_string(),
            encrypt_tab: "Encrypt".to_string(),
            decrypt_tab: "Decrypt".to_string(),
            input_file_label: "Input File:".to_string(),
            output_file_label: "Output File:".to_string(),
            browse_button: "Browse...".to_string(),
            seed_label: "Password/Seed:".to_string(),
            seed_tooltip: "Enter a secure password or seed phrase for encryption".to_string(),
            chunk_size_label: "Chunk Size:".to_string(),
            unit_label: "Unit:".to_string(),
            kilobyte: "KB".to_string(),
            megabyte: "MB".to_string(),
            gigabyte: "GB".to_string(),
            encrypt_button: "Encrypt File".to_string(),
            decrypt_button: "Decrypt File".to_string(),
            progress_label: "Progress:".to_string(),
            split_large_files: "Split large files into chunks".to_string(),
            language_label: "Language:".to_string(),
            english: "English".to_string(),
            turkish: "Turkish".to_string(),
            success_encrypt: "File encrypted successfully!".to_string(),
            success_decrypt: "File decrypted successfully!".to_string(),
            error_no_input: "Please select an input file.".to_string(),
            error_no_output: "Please select an output file.".to_string(),
            error_no_seed: "Please enter a password/seed.".to_string(),
            error_processing: "Error processing file: ".to_string(),
            error_title: "Error".to_string(),
            success_title: "Success".to_string(),
            large_file_prompt: "Large file detected! Do you want to split it into parts?".to_string(),
            yes: "Yes".to_string(),
            no: "No".to_string(),
            cancel: "Cancel".to_string(),
        }
    }

    pub fn turkish() -> Self {
        Translations {
            window_title: "Aşkın Şifreleme Aracı".to_string(),
            encrypt_tab: "Şifrele".to_string(),
            decrypt_tab: "Şifre Çöz".to_string(),
            input_file_label: "Girdi Dosyası:".to_string(),
            output_file_label: "Çıktı Dosyası:".to_string(),
            browse_button: "Gözat...".to_string(),
            seed_label: "Parola/Tohum:".to_string(),
            seed_tooltip: "Şifreleme için güvenli bir parola veya tohum cümlesi girin".to_string(),
            chunk_size_label: "Parça Boyutu:".to_string(),
            unit_label: "Birim:".to_string(),
            kilobyte: "KB".to_string(),
            megabyte: "MB".to_string(),
            gigabyte: "GB".to_string(),
            encrypt_button: "Dosyayı Şifrele".to_string(),
            decrypt_button: "Şifreyi Çöz".to_string(),
            progress_label: "İlerleme:".to_string(),
            split_large_files: "Büyük dosyaları parçalara böl".to_string(),
            language_label: "Dil:".to_string(),
            english: "İngilizce".to_string(),
            turkish: "Türkçe".to_string(),
            success_encrypt: "Dosya başarıyla şifrelendi!".to_string(),
            success_decrypt: "Dosyanın şifresi başarıyla çözüldü!".to_string(),
            error_no_input: "Lütfen bir girdi dosyası seçin.".to_string(),
            error_no_output: "Lütfen bir çıktı dosyası seçin.".to_string(),
            error_no_seed: "Lütfen bir parola/tohum girin.".to_string(),
            error_processing: "Dosya işlenirken hata oluştu: ".to_string(),
            error_title: "Hata".to_string(),
            success_title: "Başarılı".to_string(),
            large_file_prompt: "Büyük dosya tespit edildi! Parçalara bölmek ister misiniz?".to_string(),
            yes: "Evet".to_string(),
            no: "Hayır".to_string(),
            cancel: "İptal".to_string(),
        }
    }
}

// ==================== APP STATE ====================

#[derive(PartialEq)]
enum Tab {
    Encrypt,
    Decrypt,
}

struct TranscendentalCryptoApp {
    // Language and translations
    current_language: String,
    translations: Translations,
    
    // UI state
    current_tab: Tab,
    
    // Input fields
    input_path: String,
    output_path: String,
    seed: String,
    chunk_size: String,
    selected_unit: SizeUnit,
    split_large_files: bool,
    
    // Process state
    is_processing: bool,
    progress: f32, // 0.0 to 1.0
    status_message: String,
}

impl Default for TranscendentalCryptoApp {
    fn default() -> Self {
        Self {
            current_language: "en".to_string(),
            translations: Translations::english(),
            current_tab: Tab::Encrypt,
            input_path: String::new(),
            output_path: String::new(),
            seed: String::new(),
            chunk_size: "64".to_string(),
            selected_unit: SizeUnit::MB,
            split_large_files: true,
            is_processing: false,
            progress: 0.0,
            status_message: String::new(),
        }
    }
}

impl eframe::App for TranscendentalCryptoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Language selector on top
            ui.horizontal(|ui| {
                ui.label(&self.translations.language_label);
                if ui.button(&self.translations.english).clicked() {
                    self.current_language = "en".to_string();
                    self.translations = Translations::english();
                }
                if ui.button(&self.translations.turkish).clicked() {
                    self.current_language = "tr".to_string();
                    self.translations = Translations::turkish();
                }
            });
            
            ui.add_space(10.0);
            
            // Tabs
            ui.horizontal(|ui| {
                if ui.button(&self.translations.encrypt_tab).clicked() {
                    self.current_tab = Tab::Encrypt;
                }
                if ui.button(&self.translations.decrypt_tab).clicked() {
                    self.current_tab = Tab::Decrypt;
                }
            });
            
            ui.separator();
            
            match self.current_tab {
                Tab::Encrypt => self.render_encrypt_tab(ui),
                Tab::Decrypt => self.render_decrypt_tab(ui),
            }
            
            ui.separator();
            
            // Progress bar
            if self.is_processing {
                ui.label(&self.translations.progress_label);
                ui.add(egui::ProgressBar::new(self.progress).show_percentage());
            }
            
            // Status message
            if !self.status_message.is_empty() {
                ui.colored_label(egui::Color32::from_rgb(100, 200, 100), &self.status_message);
            }
        });
    }
}

impl TranscendentalCryptoApp {
    fn render_encrypt_tab(&mut self, ui: &mut egui::Ui) {
        // Input file selection
        ui.horizontal(|ui| {
            ui.label(&self.translations.input_file_label);
            ui.text_edit_singleline(&mut self.input_path);
            if ui.button(&self.translations.browse_button).clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.input_path = path.display().to_string();
                    
                    // Suggest output file
                    let path = Path::new(&self.input_path);
                    if let Some(parent) = path.parent() {
                        if let Some(file_name) = path.file_name() {
                            let mut output_name = file_name.to_string_lossy().to_string();
                            output_name.push_str(".enc");
                            let output_path = parent.join(output_name);
                            self.output_path = output_path.display().to_string();
                        }
                    }
                }
            }
        });
        
        // Output file selection
        ui.horizontal(|ui| {
            ui.label(&self.translations.output_file_label);
            ui.text_edit_singleline(&mut self.output_path);
            if ui.button(&self.translations.browse_button).clicked() {
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    self.output_path = path.display().to_string();
                }
            }
        });
        
        // Password/seed
        ui.horizontal(|ui| {
            ui.label(&self.translations.seed_label);
            ui.add(egui::TextEdit::singleline(&mut self.seed).password(true));
        });
        
        // Chunk size and unit
        ui.horizontal(|ui| {
            ui.label(&self.translations.chunk_size_label);
            ui.add(egui::TextEdit::singleline(&mut self.chunk_size).desired_width(60.0));
            
            ui.label(&self.translations.unit_label);
            egui::ComboBox::from_label("")
                .selected_text(match self.selected_unit {
                    SizeUnit::KB => &self.translations.kilobyte,
                    SizeUnit::MB => &self.translations.megabyte,
                    SizeUnit::GB => &self.translations.gigabyte,
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.selected_unit, SizeUnit::KB, &self.translations.kilobyte);
                    ui.selectable_value(&mut self.selected_unit, SizeUnit::MB, &self.translations.megabyte);
                    ui.selectable_value(&mut self.selected_unit, SizeUnit::GB, &self.translations.gigabyte);
                });
        });
        
        // Split large files checkbox
        ui.checkbox(&mut self.split_large_files, &self.translations.split_large_files);
        
        ui.add_space(10.0);
        
        // Encrypt button
        if ui.add_enabled(!self.is_processing, egui::Button::new(&self.translations.encrypt_button)).clicked() {
            self.encrypt_file();
        }
    }
    
    fn render_decrypt_tab(&mut self, ui: &mut egui::Ui) {
        // Input file selection
        ui.horizontal(|ui| {
            ui.label(&self.translations.input_file_label);
            ui.text_edit_singleline(&mut self.input_path);
            if ui.button(&self.translations.browse_button).clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.input_path = path.display().to_string();
                    
                    // Suggest output file
                    let path = Path::new(&self.input_path);
                    if let Some(parent) = path.parent() {
                        if let Some(file_name) = path.file_name() {
                            let mut output_name = file_name.to_string_lossy().to_string();
                            if output_name.ends_with(".enc") {
                                output_name = output_name[..output_name.len()-4].to_string();
                            } else {
                                output_name.push_str(".dec");
                            }
                            let output_path = parent.join(output_name);
                            self.output_path = output_path.display().to_string();
                        }
                    }
                }
            }
        });
        
        // Output file selection
        ui.horizontal(|ui| {
            ui.label(&self.translations.output_file_label);
            ui.text_edit_singleline(&mut self.output_path);
            if ui.button(&self.translations.browse_button).clicked() {
                if let Some(path) = rfd::FileDialog::new().save_file() {
                    self.output_path = path.display().to_string();
                }
            }
        });
        
        // Password/seed
        ui.horizontal(|ui| {
            ui.label(&self.translations.seed_label);
            ui.add(egui::TextEdit::singleline(&mut self.seed).password(true));
        });
        
        ui.add_space(10.0);
        
        // Decrypt button
        if ui.add_enabled(!self.is_processing, egui::Button::new(&self.translations.decrypt_button)).clicked() {
            self.decrypt_file();
        }
    }
    
    fn encrypt_file(&mut self) {
        // Validate inputs
        if self.input_path.is_empty() {
            self.status_message = self.translations.error_no_input.clone();
            return;
        }
        
        if self.output_path.is_empty() {
            self.status_message = self.translations.error_no_output.clone();
            return;
        }
        
        if self.seed.is_empty() {
            self.status_message = self.translations.error_no_seed.clone();
            return;
        }
        
        let chunk_size = match self.chunk_size.parse::<u64>() {
            Ok(size) => size,
            Err(_) => {
                self.status_message = "Invalid chunk size. Using default 64.".to_string();
                64
            }
        };
        
        let input_path = PathBuf::from(&self.input_path);
        let output_path = PathBuf::from(&self.output_path);
        
        // Reset progress and set state
        self.progress = 0.0;
        self.is_processing = true;
        self.status_message = "Processing...".to_string();
        
        // Create thread-safe reference to self for the thread
        let progress = Arc::new(Mutex::new(&mut self.progress));
        let status = Arc::new(Mutex::new(&mut self.status_message));
        let is_processing = Arc::new(Mutex::new(&mut self.is_processing));
        
        // Get all the values we need to move to the thread
        let seed_clone = self.seed.clone();
        let unit = self.selected_unit;
        
        // Start encryption in a separate thread
        thread::spawn(move || {
            let result = encrypt_file(&input_path, &output_path, &seed_clone, chunk_size, unit, 
                |processed, total| {
                    if let Ok(mut progress_guard) = progress.lock() {
                        **progress_guard = processed as f32 / total as f32;
                    }
                });
            
            // Update status when done
            if let Ok(mut is_processing_guard) = is_processing.lock() {
                **is_processing_guard = false;
            }
            
            if let Ok(mut status_guard) = status.lock() {
                match result {
                    Ok(_) => {
                        **status_guard = "File encrypted successfully!".to_string();
                    },
                    Err(e) => {
                        **status_guard = format!("Error: {}", e);
                    }
                }
            }
        });
    }
    
    fn decrypt_file(&mut self) {
        // Validate inputs
        if self.input_path.is_empty() {
            self.status_message = self.translations.error_no_input.clone();
            return;
        }
        
        if self.output_path.is_empty() {
            self.status_message = self.translations.error_no_output.clone();
            return;
        }
        
        if self.seed.is_empty() {
            self.status_message = self.translations.error_no_seed.clone();
            return;
        }
        
        let input_path = PathBuf::from(&self.input_path);
        let output_path = PathBuf::from(&self.output_path);
        
        // Reset progress and set state
        self.progress = 0.0;
        self.is_processing = true;
        self.status_message = "Processing...".to_string();
        
        // Create thread-safe reference to self for the thread
        let progress = Arc::new(Mutex::new(&mut self.progress));
        let status = Arc::new(Mutex::new(&mut self.status_message));
        let is_processing = Arc::new(Mutex::new(&mut self.is_processing));
        
        // Get all the values we need to move to the thread
        let seed_clone = self.seed.clone();
        
        // Start decryption in a separate thread
        thread::spawn(move || {
            let result = decrypt_file(&input_path, &output_path, &seed_clone, 64, SizeUnit::MB, 
                |processed, total| {
                    if let Ok(mut progress_guard) = progress.lock() {
                        **progress_guard = processed as f32 / total as f32;
                    }
                });
            
            // Update status when done
            if let Ok(mut is_processing_guard) = is_processing.lock() {
                **is_processing_guard = false;
            }
            
            if let Ok(mut status_guard) = status.lock() {
                match result {
                    Ok(_) => {
                        **status_guard = "File decrypted successfully!".to_string();
                    },
                    Err(e) => {
                        **status_guard = format!("Error: {}", e);
                    }
                }
            }
        });
    }
}

// ==================== ENCRYPTION FUNCTIONS ====================

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

// ==================== MAIN FUNCTION ====================

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(500.0, 400.0)),
        icon_data: None, // We could add an icon here if needed
        ..Default::default()
    };
    
    eframe::run_native(
        "Transcendental Encryption Tool",
        options,
        Box::new(|_cc| Box::new(TranscendentalCryptoApp::default())),
    )
    .expect("Failed to start application");
}