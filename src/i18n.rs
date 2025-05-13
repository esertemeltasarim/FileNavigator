use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

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
    pub file_dialog_filter: String,
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
            file_dialog_filter: "All Files (*.*)\0*.*\0".to_string(),
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
            file_dialog_filter: "Tüm Dosyalar (*.*)\0*.*\0".to_string(),
        }
    }

    pub fn load_or_default() -> Self {
        let config_dir = match dirs::config_dir() {
            Some(dir) => dir.join("transcendental_crypto"),
            None => return Self::default(),
        };

        if !config_dir.exists() {
            if let Err(_) = fs::create_dir_all(&config_dir) {
                return Self::default();
            }
        }

        let config_file = config_dir.join("config.json");
        match fs::read_to_string(&config_file) {
            Ok(content) => {
                match serde_json::from_str::<Config>(&content) {
                    Ok(config) => {
                        if config.language == "tr" {
                            Self::turkish()
                        } else {
                            Self::english()
                        }
                    }
                    Err(_) => Self::default(),
                }
            }
            Err(_) => Self::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub language: String,
}

pub fn save_language_preference(language: &str) -> io::Result<()> {
    let config_dir = match dirs::config_dir() {
        Some(dir) => dir.join("transcendental_crypto"),
        None => return Err(io::Error::new(io::ErrorKind::NotFound, "Could not find config directory")),
    };

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
    }

    let config_file = config_dir.join("config.json");
    let config = Config {
        language: language.to_string(),
    };

    let content = serde_json::to_string_pretty(&config)?;
    fs::write(config_file, content)?;

    Ok(())
}

pub fn get_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("transcendental_crypto").join("config.json"))
}
