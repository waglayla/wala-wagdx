use crate::imports::*;
use crate::error::Error;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

#[derive(PartialEq, Eq)]
pub struct StorageFolder {
  pub path: PathBuf,
  pub folder_size: u64,
  pub folder_size_string: String,
  pub confirm_deletion: bool,
}

#[derive(Default, Debug, Clone)]
pub struct StorageUpdateOptions {
  pub update_if_not_present: bool,
  pub delay: Option<Duration>,
}

impl StorageUpdateOptions {
  pub fn if_not_present(mut self) -> Self {
    self.update_if_not_present = true;
    self
  }

  pub fn with_delay(mut self, delay: Duration) -> Self {
    self.delay = Some(delay);
    self
  }
}

#[derive(Default, Clone)]
pub struct Storage {
  pub folder: Arc<Mutex<Option<StorageFolder>>>,
  pub storage_root: Arc<Mutex<Option<PathBuf>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Storage {
  pub fn new() -> Self {
    Self {
      folder: Arc::new(Mutex::new(None)),
      storage_root: Arc::new(Mutex::new(None)),
    }
  }

  pub fn track_storage_root(&self, storage_root: Option<&str>) {
    *self.storage_root.lock().unwrap() = storage_root.map(PathBuf::from);
    self.update();
  }

  pub fn storage_root(&self) -> PathBuf {
    self.storage_root
      .lock()
      .unwrap()
      .clone()
      .unwrap_or_else(|| PathBuf::from("./app_data")) // Default path
  }

  pub fn update(&self) -> Result<()> {
    let app_dir = self.storage_root();
    if !app_dir.exists() {
      return Err(Error::Custom("Storage root does not exist".to_string()));
    }

    let path = app_dir.join("data-mainnet");
    if path.exists() && path.is_dir() {
      let mut folder_size = 0;
      for entry in WalkDir::new(&path).into_iter().flatten() {
        folder_size += entry
          .metadata()
          .map(|metadata| metadata.len())
          .unwrap_or_default();
      }

      self.update_folder_size(folder_size, path);
      Ok(())
    } else {
      Err(Error::Custom("Data directory not found".to_string()))
    }
  }

  fn update_folder_size(&self, folder_size: u64, path: PathBuf) {
    let folder_size_string = format!("{:.2} MB", folder_size as f64 / 1_000_000.0);
    
    let mut folder_lock = self.folder.lock().unwrap();
    *folder_lock = Some(StorageFolder {
      path,
      folder_size,
      folder_size_string,
      confirm_deletion: false,
    });
  }

  pub fn get_folder(&self) -> Result<Option<PathBuf>> {
    self.folder
      .lock()
      .map_err(|e| Error::Custom(format!("Lock error: {}", e)))
      .map(|guard| guard.as_ref().map(|f| f.path.clone()))
  }

  pub fn remove(&self) -> Result<()> {
    if let Ok(Some(path)) = self.get_folder() {
      if path.exists() {
        println!("Removing storage folder: {:?}", path.display());
        match std::fs::remove_dir_all(&path) {
          Ok(_) => {
            println!("Storage folder removed: {:?}", path.display());
            self.update();
            Ok(())
          },
          Err(e) => {
            println!("Error removing storage folder: {:?}", e);
            // Use the From implementation for IoError
            Err(Error::from(e))
          }
        }
      } else {
        println!("Folder not found: {}", path.display());
        Ok(())
      }
  } else {
    Ok(())
  }
}

  pub fn clear_settings(&self) {
    if let Some(folder) = &mut *self.folder.lock().unwrap() {
      folder.confirm_deletion = false;
    }
  }

  // UI rendering methods can stay mostly the same, just remove async-specific code
  pub fn render(&self, ui: &mut egui::Ui) {
    if let Some(folder) = self.folder.lock().unwrap().as_ref() {
      ui.vertical_centered(|ui| {
        egui::CollapsingHeader::new("Storage")
          .default_open(true)
          .show(ui, |ui| {
            ui.vertical(|ui| {
              ui.label(format!("Storage size: {}", folder.folder_size_string));
            });
          });
      });
    }
  }

  pub fn render_settings(&self, core: &mut Core, ui: &mut Ui) {
    let mut node_folder = self.folder.lock().unwrap();
    if let Some(folder) = node_folder.as_mut() {
      ui.vertical_centered(|ui| {
        CollapsingHeader::new(i18n("Storage"))
        .default_open(false)
        .show(ui, |ui| {
          ui.vertical(|ui| {
            let StorageFolder {
              folder_size_string,
              path,
              confirm_deletion,
              ..
            } = folder;

            CollapsingHeader::new(format!("{}: {folder_size_string}", "MAINNET".to_string()))
            .default_open(false)
            .show(ui, |ui| {
              let is_running = core.settings.node.node_kind.is_local();

              ui.horizontal(|ui|{
                if ui.medium_button(i18n("Open Data Folder")).clicked() {
                  if let Err(err) = open::that(path) {
                    manager().error(format!("Error opening folder: {:?}", err));
                  }
                }
                if ui.medium_button_enabled(!is_running && !*confirm_deletion, i18n("Delete Data Folder")).clicked() {
                  *confirm_deletion = true;
                }
              });

              if is_running {
                ui.label(i18n("Cannot delete data folder while the node is running"));
                ui.label(i18n("Please set node to 'Disabled' to delete the data folder"));
              }

              if *confirm_deletion {
                ui.add_sized(vec2(260.,4.), Separator::default());
                ui.label(i18n("This action will erase WagLayla database and logs"));
                ui.label("");
                ui.colored_label(theme_color().alert_color, i18n("Please Confirm Deletion"));
                if let Some(response) = ui.confirm_widget_labels("Apply", "Cancel") {
                  match response {
                    Confirm::Yes => {
                      *confirm_deletion = false;
                      self.remove();
                    },
                    Confirm::No => {
                      *confirm_deletion = false;
                    }
                  }
                }
                ui.add_sized(vec2(260.,4.), Separator::default());
              }
            });
          });
        });
      });
    }
  }
}