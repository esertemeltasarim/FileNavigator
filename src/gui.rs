use crate::encryption::{encrypt_file, decrypt_file, split_and_encrypt, detect_parts, SizeUnit};
use crate::i18n::{Translations, save_language_preference};
use native_windows_derive::NwgUi;
use native_windows_gui as nwg;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

// GUI resource IDs
const IDC_STATIC_INPUT_FILE: usize = 1001;
const IDC_EDIT_INPUT_FILE: usize = 1002;
const IDC_BUTTON_BROWSE_INPUT: usize = 1003;
const IDC_STATIC_OUTPUT_FILE: usize = 1004;
const IDC_EDIT_OUTPUT_FILE: usize = 1005;
const IDC_BUTTON_BROWSE_OUTPUT: usize = 1006;
const IDC_STATIC_SEED: usize = 1007;
const IDC_EDIT_SEED: usize = 1008;
const IDC_STATIC_CHUNK_SIZE: usize = 1009;
const IDC_EDIT_CHUNK_SIZE: usize = 1010;
const IDC_STATIC_UNIT: usize = 1011;
const IDC_COMBO_UNIT: usize = 1012;
const IDC_BUTTON_ENCRYPT: usize = 1013;
const IDC_BUTTON_DECRYPT: usize = 1014;
const IDC_PROGRESS: usize = 1015;
const IDC_STATIC_PROGRESS: usize = 1016;
const IDC_CHECK_SPLIT_FILES: usize = 1017;
const IDC_TAB_CONTROL: usize = 1018;
const IDC_STATIC_LANGUAGE: usize = 1019;
const IDC_COMBO_LANGUAGE: usize = 1020;

#[derive(Default, NwgUi)]
pub struct TranscendentalCryptoApp {
    // Application data
    translations: RefCell<Translations>,
    current_language: RefCell<String>,
    
    // Main window
    #[nwg_control(size: (500, 400), position: (300, 300), title: "Transcendental Encryption Tool", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [TranscendentalCryptoApp::exit])]
    window: nwg::Window,

    #[nwg_control(parent: window, position: (10, 10), size: (480, 350), flags: "VISIBLE")]
    tab_control: nwg::TabsContainer,

    // Encrypt tab
    #[nwg_control(parent: tab_control, text: "Encrypt")]
    tab_encrypt: nwg::Tab,

    // Input file for encryption
    #[nwg_control(parent: tab_encrypt, text: "Input File:", position: (10, 20), size: (90, 20))]
    static_input_file_enc: nwg::Label,
    
    #[nwg_control(parent: tab_encrypt, text: "", position: (100, 20), size: (290, 20))]
    edit_input_file_enc: nwg::TextInput,
    
    #[nwg_control(parent: tab_encrypt, text: "Browse...", position: (400, 20), size: (80, 20))]
    #[nwg_events(OnButtonClick: [TranscendentalCryptoApp::browse_input_enc])]
    button_browse_input_enc: nwg::Button,
    
    // Output file for encryption
    #[nwg_control(parent: tab_encrypt, text: "Output File:", position: (10, 50), size: (90, 20))]
    static_output_file_enc: nwg::Label,
    
    #[nwg_control(parent: tab_encrypt, text: "", position: (100, 50), size: (290, 20))]
    edit_output_file_enc: nwg::TextInput,
    
    #[nwg_control(parent: tab_encrypt, text: "Browse...", position: (400, 50), size: (80, 20))]
    #[nwg_events(OnButtonClick: [TranscendentalCryptoApp::browse_output_enc])]
    button_browse_output_enc: nwg::Button,
    
    // Seed/password for encryption
    #[nwg_control(parent: tab_encrypt, text: "Password/Seed:", position: (10, 80), size: (90, 20))]
    static_seed_enc: nwg::Label,
    
    #[nwg_control(parent: tab_encrypt, text: "", position: (100, 80), size: (290, 20), flags: "VISIBLE|PASSWORD")]
    edit_seed_enc: nwg::TextInput,
    
    // Chunk size for encryption
    #[nwg_control(parent: tab_encrypt, text: "Chunk Size:", position: (10, 110), size: (90, 20))]
    static_chunk_size_enc: nwg::Label,
    
    #[nwg_control(parent: tab_encrypt, text: "64", position: (100, 110), size: (80, 20))]
    edit_chunk_size_enc: nwg::TextInput,
    
    // Unit selection for encryption
    #[nwg_control(parent: tab_encrypt, text: "Unit:", position: (190, 110), size: (50, 20))]
    static_unit_enc: nwg::Label,
    
    #[nwg_control(parent: tab_encrypt, position: (240, 110), size: (80, 20), collection: vec!["KB", "MB", "GB"], selected_index: Some(1))]
    combo_unit_enc: nwg::ComboBox<&'static str>,
    
    // Split large files checkbox
    #[nwg_control(parent: tab_encrypt, text: "Split large files into chunks", position: (100, 140), size: (200, 20), check_state: nwg::CheckBoxState::Checked)]
    check_split_files_enc: nwg::CheckBox,
    
    // Encrypt button
    #[nwg_control(parent: tab_encrypt, text: "Encrypt File", position: (200, 170), size: (100, 30))]
    #[nwg_events(OnButtonClick: [TranscendentalCryptoApp::encrypt])]
    button_encrypt: nwg::Button,
    
    // Progress indicators for encryption
    #[nwg_control(parent: tab_encrypt, text: "Progress:", position: (10, 210), size: (90, 20))]
    static_progress_enc: nwg::Label,
    
    #[nwg_control(parent: tab_encrypt, position: (100, 210), size: (380, 20))]
    progress_enc: nwg::ProgressBar,

    // Decrypt tab
    #[nwg_control(parent: tab_control, text: "Decrypt")]
    tab_decrypt: nwg::Tab,

    // Input file for decryption
    #[nwg_control(parent: tab_decrypt, text: "Input File:", position: (10, 20), size: (90, 20))]
    static_input_file_dec: nwg::Label,
    
    #[nwg_control(parent: tab_decrypt, text: "", position: (100, 20), size: (290, 20))]
    edit_input_file_dec: nwg::TextInput,
    
    #[nwg_control(parent: tab_decrypt, text: "Browse...", position: (400, 20), size: (80, 20))]
    #[nwg_events(OnButtonClick: [TranscendentalCryptoApp::browse_input_dec])]
    button_browse_input_dec: nwg::Button,
    
    // Output file for decryption
    #[nwg_control(parent: tab_decrypt, text: "Output File:", position: (10, 50), size: (90, 20))]
    static_output_file_dec: nwg::Label,
    
    #[nwg_control(parent: tab_decrypt, text: "", position: (100, 50), size: (290, 20))]
    edit_output_file_dec: nwg::TextInput,
    
    #[nwg_control(parent: tab_decrypt, text: "Browse...", position: (400, 50), size: (80, 20))]
    #[nwg_events(OnButtonClick: [TranscendentalCryptoApp::browse_output_dec])]
    button_browse_output_dec: nwg::Button,
    
    // Seed/password for decryption
    #[nwg_control(parent: tab_decrypt, text: "Password/Seed:", position: (10, 80), size: (90, 20))]
    static_seed_dec: nwg::Label,
    
    #[nwg_control(parent: tab_decrypt, text: "", position: (100, 80), size: (290, 20), flags: "VISIBLE|PASSWORD")]
    edit_seed_dec: nwg::TextInput,

    // Decrypt button
    #[nwg_control(parent: tab_decrypt, text: "Decrypt File", position: (200, 170), size: (100, 30))]
    #[nwg_events(OnButtonClick: [TranscendentalCryptoApp::decrypt])]
    button_decrypt: nwg::Button,
    
    // Progress indicators for decryption
    #[nwg_control(parent: tab_decrypt, text: "Progress:", position: (10, 210), size: (90, 20))]
    static_progress_dec: nwg::Label,
    
    #[nwg_control(parent: tab_decrypt, position: (100, 210), size: (380, 20))]
    progress_dec: nwg::ProgressBar,

    // Settings area (common for both tabs)
    #[nwg_control(parent: window, text: "Language:", position: (10, 365), size: (70, 20))]
    static_language: nwg::Label,
    
    #[nwg_control(parent: window, position: (80, 365), size: (100, 20), collection: vec!["English", "Turkish"], selected_index: Some(0))]
    #[nwg_events(OnComboxBoxSelection: [TranscendentalCryptoApp::change_language])]
    combo_language: nwg::ComboBox<&'static str>,

    // Open file dialogs
    #[nwg_resource(title: "Select Input File", action: nwg::FileDialogAction::Open, filters: "All Files (*.*)|*.*")]
    file_dialog_input: nwg::FileDialog,

    #[nwg_resource(title: "Select Output File", action: nwg::FileDialogAction::Save, filters: "All Files (*.*)|*.*")]
    file_dialog_output: nwg::FileDialog,

    // Message dialogs
    #[nwg_resource(title: "Error", buttons: vec![nwg::MessageDialogButtons::Ok], icons: nwg::MessageDialogIcon::Error)]
    error_dialog: nwg::MessageDialog,

    #[nwg_resource(title: "Success", buttons: vec![nwg::MessageDialogButtons::Ok], icons: nwg::MessageDialogIcon::Info)]
    success_dialog: nwg::MessageDialog,

    #[nwg_resource(title: "Confirmation", buttons: vec![nwg::MessageDialogButtons::YesNo], icons: nwg::MessageDialogIcon::Question)]
    confirmation_dialog: nwg::MessageDialog,
}

impl TranscendentalCryptoApp {
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

    fn browse_input_enc(&self) {
        if self.file_dialog_input.run(Some(&self.window)) {
            if let Ok(path) = self.file_dialog_input.get_selected_item() {
                self.edit_input_file_enc.set_text(&path.to_string_lossy());
                
                // Suggest output file name with .enc extension
                if let Some(parent) = path.parent() {
                    if let Some(file_name) = path.file_name() {
                        let mut output_name = file_name.to_string_lossy().to_string();
                        output_name.push_str(".enc");
                        let output_path = parent.join(output_name);
                        self.edit_output_file_enc.set_text(&output_path.to_string_lossy());
                    }
                }
            }
        }
    }

    fn browse_output_enc(&self) {
        if self.file_dialog_output.run(Some(&self.window)) {
            if let Ok(path) = self.file_dialog_output.get_selected_item() {
                self.edit_output_file_enc.set_text(&path.to_string_lossy());
            }
        }
    }

    fn browse_input_dec(&self) {
        if self.file_dialog_input.run(Some(&self.window)) {
            if let Ok(path) = self.file_dialog_input.get_selected_item() {
                self.edit_input_file_dec.set_text(&path.to_string_lossy());
                
                // Suggest output file name without .enc extension
                if let Some(parent) = path.parent() {
                    if let Some(file_name) = path.file_name() {
                        let mut output_name = file_name.to_string_lossy().to_string();
                        if output_name.ends_with(".enc") {
                            output_name = output_name[..output_name.len()-4].to_string();
                        } else {
                            output_name.push_str(".dec");
                        }
                        let output_path = parent.join(output_name);
                        self.edit_output_file_dec.set_text(&output_path.to_string_lossy());
                    }
                }
            }
        }
    }

    fn browse_output_dec(&self) {
        if self.file_dialog_output.run(Some(&self.window)) {
            if let Ok(path) = self.file_dialog_output.get_selected_item() {
                self.edit_output_file_dec.set_text(&path.to_string_lossy());
            }
        }
    }

    fn encrypt(&self) {
        // Get input values
        let input_file = self.edit_input_file_enc.text();
        let output_file = self.edit_output_file_enc.text();
        let seed = self.edit_seed_enc.text();
        let chunk_size_text = self.edit_chunk_size_enc.text();
        let split_large_files = self.check_split_files_enc.check_state() == nwg::CheckBoxState::Checked;
        
        // Validate inputs
        let translations = self.translations.borrow();
        
        if input_file.is_empty() {
            self.error_dialog.set_title(&translations.error_title);
            self.error_dialog.set_text(&translations.error_no_input);
            self.error_dialog.run(Some(&self.window));
            return;
        }
        
        if output_file.is_empty() {
            self.error_dialog.set_title(&translations.error_title);
            self.error_dialog.set_text(&translations.error_no_output);
            self.error_dialog.run(Some(&self.window));
            return;
        }
        
        if seed.is_empty() {
            self.error_dialog.set_title(&translations.error_title);
            self.error_dialog.set_text(&translations.error_no_seed);
            self.error_dialog.run(Some(&self.window));
            return;
        }
        
        let chunk_size = match chunk_size_text.parse::<u64>() {
            Ok(size) => size,
            Err(_) => {
                self.error_dialog.set_title(&translations.error_title);
                self.error_dialog.set_text("Invalid chunk size. Using default 64.");
                self.error_dialog.run(Some(&self.window));
                64
            }
        };
        
        let unit = match self.combo_unit_enc.selection_string() {
            Some(unit_str) => match unit_str {
                "KB" => SizeUnit::KB,
                "GB" => SizeUnit::GB,
                _ => SizeUnit::MB,
            },
            None => SizeUnit::MB,
        };
        
        let input_path = PathBuf::from(&input_file);
        let output_path = PathBuf::from(&output_file);
        
        // Check if file is large and should be split
        let large_file_threshold = 10 * 1024 * 1024; // 10 MB
        
        match std::fs::metadata(&input_path) {
            Ok(meta) => {
                let file_size = meta.len();
                let engine_chunk_size = match unit {
                    SizeUnit::KB => chunk_size * 1024,
                    SizeUnit::MB => chunk_size * 1024 * 1024,
                    SizeUnit::GB => chunk_size * 1024 * 1024 * 1024,
                };
                
                if file_size > large_file_threshold && file_size as u64 > engine_chunk_size && split_large_files {
                    self.confirmation_dialog.set_title(&translations.large_file_prompt);
                    self.confirmation_dialog.set_text(&translations.large_file_prompt);
                    
                    if self.confirmation_dialog.run(Some(&self.window)) == nwg::MessageDialogResult::Yes {
                        // Split and encrypt
                        self.process_encryption(input_path, output_path, &seed, chunk_size, unit, true);
                    } else {
                        // Encrypt without splitting
                        self.process_encryption(input_path, output_path, &seed, chunk_size, unit, false);
                    }
                } else {
                    // Regular encryption
                    self.process_encryption(input_path, output_path, &seed, chunk_size, unit, false);
                }
            },
            Err(e) => {
                self.error_dialog.set_title(&translations.error_title);
                self.error_dialog.set_text(&format!("{}{}", translations.error_processing, e));
                self.error_dialog.run(Some(&self.window));
                return;
            }
        }
    }
    
    fn process_encryption(&self, input_path: PathBuf, output_path: PathBuf, seed: &str, chunk_size: u64, unit: SizeUnit, split: bool) {
        // Reset progress bar
        self.progress_enc.set_pos(0);
        
        // Setup progress tracking
        let window_handle = self.window.handle.clone();
        let progress_handle = self.progress_enc.handle.clone();
        let success_dialog = self.success_dialog.clone();
        let error_dialog = self.error_dialog.clone();
        let translations = self.translations.borrow().clone();
        
        let progress_callback = Arc::new(Mutex::new(move |processed: u64, total: u64| {
            if total > 0 {
                let percentage = ((processed as f64 / total as f64) * 100.0) as u32;
                nwg::dispatch_thread_events_with_callback(&window_handle, move |_| {
                    nwg::ProgressBar::set_pos(&progress_handle, percentage);
                });
            }
        }));
        
        // Create thread for encryption
        let seed_clone = seed.to_string();
        thread::spawn(move || {
            let result = if split {
                let output_dir = output_path.parent().unwrap_or(Path::new(".")).to_path_buf();
                split_and_encrypt(
                    &input_path, 
                    &output_dir, 
                    &seed_clone, 
                    chunk_size, 
                    unit,
                    |processed, total| {
                        let callback = progress_callback.lock().unwrap();
                        callback(processed, total);
                    }
                )
            } else {
                encrypt_file(
                    &input_path, 
                    &output_path, 
                    &seed_clone, 
                    chunk_size, 
                    unit,
                    |processed, total| {
                        let callback = progress_callback.lock().unwrap();
                        callback(processed, total);
                    }
                ).map(|_| vec![])
            };
            
            nwg::dispatch_thread_events_with_callback(&window_handle, move |_| {
                match result {
                    Ok(_) => {
                        success_dialog.set_title(&translations.success_title);
                        success_dialog.set_text(&translations.success_encrypt);
                        success_dialog.run(None);
                    }
                    Err(e) => {
                        error_dialog.set_title(&translations.error_title);
                        error_dialog.set_text(&format!("{}{}", translations.error_processing, e));
                        error_dialog.run(None);
                    }
                }
            });
        });
    }

    fn decrypt(&self) {
        // Get input values
        let input_file = self.edit_input_file_dec.text();
        let output_file = self.edit_output_file_dec.text();
        let seed = self.edit_seed_dec.text();
        
        // Validate inputs
        let translations = self.translations.borrow();
        
        if input_file.is_empty() {
            self.error_dialog.set_title(&translations.error_title);
            self.error_dialog.set_text(&translations.error_no_input);
            self.error_dialog.run(Some(&self.window));
            return;
        }
        
        if output_file.is_empty() {
            self.error_dialog.set_title(&translations.error_title);
            self.error_dialog.set_text(&translations.error_no_output);
            self.error_dialog.run(Some(&self.window));
            return;
        }
        
        if seed.is_empty() {
            self.error_dialog.set_title(&translations.error_title);
            self.error_dialog.set_text(&translations.error_no_seed);
            self.error_dialog.run(Some(&self.window));
            return;
        }
        
        let input_path = PathBuf::from(&input_file);
        let output_path = PathBuf::from(&output_file);
        
        // Reset progress bar
        self.progress_dec.set_pos(0);
        
        // Check if this is a split file
        let is_part = match detect_parts(&input_path) {
            Ok(parts) => !parts.is_empty(),
            Err(_) => false,
        };
        
        if is_part {
            self.process_decryption_parts(input_path, output_path, &seed);
        } else {
            self.process_decryption(input_path, output_path, &seed);
        }
    }
    
    fn process_decryption(&self, input_path: PathBuf, output_path: PathBuf, seed: &str) {
        // Setup progress tracking
        let window_handle = self.window.handle.clone();
        let progress_handle = self.progress_dec.handle.clone();
        let success_dialog = self.success_dialog.clone();
        let error_dialog = self.error_dialog.clone();
        let translations = self.translations.borrow().clone();
        
        let progress_callback = Arc::new(Mutex::new(move |processed: u64, total: u64| {
            if total > 0 {
                let percentage = ((processed as f64 / total as f64) * 100.0) as u32;
                nwg::dispatch_thread_events_with_callback(&window_handle, move |_| {
                    nwg::ProgressBar::set_pos(&progress_handle, percentage);
                });
            }
        }));
        
        // Create thread for decryption
        let seed_clone = seed.to_string();
        thread::spawn(move || {
            let result = decrypt_file(
                &input_path, 
                &output_path, 
                &seed_clone, 
                64, // Default chunk size 
                SizeUnit::MB, // Default unit
                |processed, total| {
                    let callback = progress_callback.lock().unwrap();
                    callback(processed, total);
                }
            );
            
            nwg::dispatch_thread_events_with_callback(&window_handle, move |_| {
                match result {
                    Ok(_) => {
                        success_dialog.set_title(&translations.success_title);
                        success_dialog.set_text(&translations.success_decrypt);
                        success_dialog.run(None);
                    }
                    Err(e) => {
                        error_dialog.set_title(&translations.error_title);
                        error_dialog.set_text(&format!("{}{}", translations.error_processing, e));
                        error_dialog.run(None);
                    }
                }
            });
        });
    }
    
    fn process_decryption_parts(&self, input_path: PathBuf, output_path: PathBuf, seed: &str) {
        // Setup progress tracking
        let window_handle = self.window.handle.clone();
        let progress_handle = self.progress_dec.handle.clone();
        let success_dialog = self.success_dialog.clone();
        let error_dialog = self.error_dialog.clone();
        let translations = self.translations.borrow().clone();
        
        let progress_callback = Arc::new(Mutex::new(move |processed: u64, total: u64| {
            if total > 0 {
                let percentage = ((processed as f64 / total as f64) * 100.0) as u32;
                nwg::dispatch_thread_events_with_callback(&window_handle, move |_| {
                    nwg::ProgressBar::set_pos(&progress_handle, percentage);
                });
            }
        }));
        
        // Create thread for decryption
        let seed_clone = seed.to_string();
        thread::spawn(move || {
            // First detect all parts
            let parts_result = detect_parts(&input_path);
            
            let result = match parts_result {
                Ok(parts) => {
                    if parts.is_empty() {
                        // Process as single file if no parts found
                        decrypt_file(
                            &input_path, 
                            &output_path, 
                            &seed_clone, 
                            64, // Default chunk size 
                            SizeUnit::MB, // Default unit
                            |processed, total| {
                                let callback = progress_callback.lock().unwrap();
                                callback(processed, total);
                            }
                        )
                    } else {
                        // Process parts
                        crate::encryption::decrypt_parts(
                            &parts,
                            &output_path,
                            &seed_clone,
                            |processed, total| {
                                let callback = progress_callback.lock().unwrap();
                                callback(processed, total);
                            }
                        )
                    }
                },
                Err(e) => Err(e),
            };
            
            nwg::dispatch_thread_events_with_callback(&window_handle, move |_| {
                match result {
                    Ok(_) => {
                        success_dialog.set_title(&translations.success_title);
                        success_dialog.set_text(&translations.success_decrypt);
                        success_dialog.run(None);
                    }
                    Err(e) => {
                        error_dialog.set_title(&translations.error_title);
                        error_dialog.set_text(&format!("{}{}", translations.error_processing, e));
                        error_dialog.run(None);
                    }
                }
            });
        });
    }
    
    fn change_language(&self) {
        let language = match self.combo_language.selection_string() {
            Some(language) => language,
            None => "English",
        };
        
        let new_translations = if language == "Turkish" {
            *self.current_language.borrow_mut() = "tr".to_string();
            Translations::turkish()
        } else {
            *self.current_language.borrow_mut() = "en".to_string();
            Translations::english()
        };
        
        // Save language preference
        let _ = save_language_preference(if language == "Turkish" { "tr" } else { "en" });
        
        // Update UI with new translations
        *self.translations.borrow_mut() = new_translations.clone();
        
        // Update window title
        self.window.set_text(&new_translations.window_title);
        
        // Update tabs
        self.tab_control.set_tab_text(0, &new_translations.encrypt_tab);
        self.tab_control.set_tab_text(1, &new_translations.decrypt_tab);
        
        // Update encrypt tab
        self.static_input_file_enc.set_text(&new_translations.input_file_label);
        self.static_output_file_enc.set_text(&new_translations.output_file_label);
        self.button_browse_input_enc.set_text(&new_translations.browse_button);
        self.button_browse_output_enc.set_text(&new_translations.browse_button);
        self.static_seed_enc.set_text(&new_translations.seed_label);
        self.static_chunk_size_enc.set_text(&new_translations.chunk_size_label);
        self.static_unit_enc.set_text(&new_translations.unit_label);
        self.button_encrypt.set_text(&new_translations.encrypt_button);
        self.static_progress_enc.set_text(&new_translations.progress_label);
        self.check_split_files_enc.set_text(&new_translations.split_large_files);
        
        // Update decrypt tab
        self.static_input_file_dec.set_text(&new_translations.input_file_label);
        self.static_output_file_dec.set_text(&new_translations.output_file_label);
        self.button_browse_input_dec.set_text(&new_translations.browse_button);
        self.button_browse_output_dec.set_text(&new_translations.browse_button);
        self.static_seed_dec.set_text(&new_translations.seed_label);
        self.button_decrypt.set_text(&new_translations.decrypt_button);
        self.static_progress_dec.set_text(&new_translations.progress_label);
        
        // Update settings area
        self.static_language.set_text(&new_translations.language_label);
        
        // Update combo box
        let kb_index = self.combo_unit_enc.collection().iter().position(|&s| s == "KB").unwrap_or(0);
        let mb_index = self.combo_unit_enc.collection().iter().position(|&s| s == "MB").unwrap_or(1);
        let gb_index = self.combo_unit_enc.collection().iter().position(|&s| s == "GB").unwrap_or(2);
        
        let mut new_items = vec![
            new_translations.kilobyte.as_str(),
            new_translations.megabyte.as_str(),
            new_translations.gigabyte.as_str(),
        ];
        
        self.combo_unit_enc.set_collection(new_items);
        
        // Update dialog titles
        self.file_dialog_input.set_filters(&new_translations.file_dialog_filter);
        self.file_dialog_output.set_filters(&new_translations.file_dialog_filter);
    }

    fn init(&self) {
        // Load saved language
        let translations = Translations::load_or_default();
        *self.translations.borrow_mut() = translations.clone();
        
        // Set initial language
        let language = if translations.window_title == Translations::turkish().window_title {
            "Turkish"
        } else {
            "English"
        };
        
        *self.current_language.borrow_mut() = if language == "Turkish" { "tr" } else { "en" }.to_string();
        
        // Set initial language in combo box
        self.combo_language.set_selection(if language == "Turkish" { 1 } else { 0 });
        
        // Update UI with translations
        self.change_language();
        
        // Initialize progress bars
        self.progress_enc.set_range(0, 100);
        self.progress_dec.set_range(0, 100);
    }
}

pub fn run_app() {
    nwg::init().expect("Failed to initialize Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    
    let app = TranscendentalCryptoApp::build_ui(Default::default()).expect("Failed to build UI");
    app.init();
    
    nwg::dispatch_thread_events();
}
