use std::env::var_os;
use std::fs::{OpenOptions, File, DirBuilder, remove_file, remove_dir_all};
use std::io::{Read, Write};
use std::path::{PathBuf, Path};

use bincode::{serialize_into, deserialize_from};
use toml;
use walkdir::{WalkDir,};

use state::{AppState, Project, Meta, Fonts, Image, Website};

///Get the app state from our last session or default 
/// if that is unavailable
pub fn get_state() -> AppState {
    if let Some(s) = try_get_cache() {
        s
    } else {
        AppState::default()
    }
}

impl AppState {
    /// use the source property of a state instance to
    /// get the current file structure/content
    pub fn update_from_source(&mut self) {
        for entry in WalkDir::new(&self.source).min_depth(1).max_depth(1) {
            if let Ok(entry) = entry {
                let name = entry.file_name();
                if name == "portfolio" {
                    self.website.update_projects_from_source(entry.path());
                } else if name == "about.md" {
                    self.website.about = content(entry.path());
                } else if name == "me.jpg" {
                    self.website.image = entry.path().to_path_buf();
                } else if name == "fonts" {
                    self.website.fonts = fonts(entry.path());
                }
            }
        }
    }

    pub fn add_project(&mut self, name: String) -> Result<(), String> {
        self.website.add_project(name);
        write_input(self)
    }
}

/// save a copy of the state to a cache file
pub fn cache_state(state: &AppState) {
    println!("cache_state");
    if let Some(f) = cache_file() {
        match serialize_into(&f, state) {
            Ok(_) => (),
            Err(e) => println!("{:?}", e),
        }
    } else {
        println!("Error getting cache file")
    }
}

/// try and get our state from a previously saved
/// cache
pub fn try_get_cache() -> Option<AppState> {
   if let Some(f) = cache_file() {
        match deserialize_from(f) {
            Ok(s) => Some(s),
            Err(_e) => None
        }
   } else {
       None
   }
}

/// Attempt to get the cache file. This will also
/// ensure that the ~/.website_builder folder is created
fn cache_file() -> Option<File> {
    let db = DirBuilder::new();
    if let Some(home) = get_user_dir() {
        let path = home.join(".site_builder");
        if !path.exists() {
            if let Err(e) = db.create(&path) {
                println!("error creating cache dir {:?}", e)
            };
        }
        match OpenOptions::new().write(true).read(true).create(true).open(path.join("cache.bincode")) {
            Ok(f) => Some(f),
            Err(e) => {
                println!("error opening cache {:?}", e);
                None
            },
        }
    } else {
        None
    }
}

fn get_user_dir() -> Option<PathBuf> {
    let arg = if cfg!(windows) {
        "USERPROFILE"
    } else {
        "HOME"
    };
    if let Some(home) = var_os(arg) {
        Some(PathBuf::from(home))
    } else {
        None
    }
}

impl Website {
    pub fn update_projects_from_source(&mut self, path: &Path) {
        // self.meta.title = path.file_name().as_str()
        let mut tmp_portfolio: Vec<Project> = vec!();
        for entry in WalkDir::new(path).min_depth(1).max_depth(1) {
            if let Ok(entry) = entry {
                match self.portfolio.binary_search_by(|p| p.path().cmp(&entry.path().to_path_buf())) {
                    Ok(idx) => {
                        let mut p = self.portfolio[idx].clone();
                        p.id = tmp_portfolio.len() as u32;
                        p.path = entry.path().to_path_buf();
                        p.update_from_source();
                        tmp_portfolio.push(p);

                    },
                    Err(_) => {
                        let mut p = Project::default();
                        p.id = tmp_portfolio.len() as u32;
                        p.path = entry.path().to_path_buf();
                        p.update_from_source();
                        tmp_portfolio.push(p);
                    }
                }
            }
        }
        self.portfolio = tmp_portfolio;
    }
    pub fn delete_project(&mut self, id: u32) {
        if let Some(ref mut p) = self.get_project(id) {
            match p.delete_files() {
                Ok(()) => {
                    self.portfolio = self.portfolio.clone().into_iter().filter(|p| p.id != id).collect();
                },
                Err(e) => println!("{:?}", e),
            }
        }
    }
}

impl Project {
    pub fn update_from_source(&mut self) {
        for entry in WalkDir::new(&self.path).min_depth(1).max_depth(1) {
            if let Ok(entry) = entry {
                let name = entry.file_name();
                println!("project file: {:?}", name);
                if name == "img" {
                    self.update_images_from_source(&entry.path());
                } else
                if name == "content.md" {
                    self.description = content(&entry.path());
                } else
                if name == "meta.toml" {
                    self.meta = meta(&entry.path());
                }
            }
        }
    }

    fn update_images_from_source(&mut self, path: &Path) {
        let mut tmp_images: Vec<Image> = vec!();
        for entry in WalkDir::new(path).min_depth(1).max_depth(1) {
            if let Ok(entry) = entry {
                if !entry.file_type().is_file() {
                    continue;
                }
                match self.images.binary_search_by(|i| i.path.cmp(&entry.path().to_path_buf())) {
                    Ok(idx) => {
                        let mut img = self.images[idx].clone();
                        img.path = entry.path().to_path_buf();
                        tmp_images.push(img);
                    },
                    Err(_) => {
                        let img = Image {
                            position: self.images.len() as u32,
                            path: entry.path().to_path_buf(),
                        };
                        tmp_images.push(img);
                    }
                }
            }
        }
        self.images = tmp_images;
        self.sort_images();
    }

    pub fn delete_files(&self) -> Result<(), String> {
        match remove_dir_all(&self.path) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Error deleting files: {:?}", e)),
        }
    }
}

fn fonts(path: &Path) -> Fonts {
    let mut ret = Fonts::default();
    for entry in WalkDir::new(path).min_depth(1).max_depth(1) {
        match entry {
            Ok(e) => {
                if let Some(n) = e.file_name().to_str() {
                    if let Some(_idx) = n.find("bold") {
                        ret.bold = Some(e.path().to_path_buf());
                    }
                } else {
                    ret.normal = Some(e.path().to_path_buf());
                }
            },
            Err(e) => println!("Error reading font file {:?}", e),
        }
    }
    ret
}

/// Parse the meta.toml file for this project
fn meta(path: &Path) -> Meta {
    let ret = Meta::default();
    let mut buf = String::new();
    if let Ok(mut f) = File::open(path) {
        if let Ok(_size) = f.read_to_string(&mut buf) {
            if let Ok(m) = toml::from_str(&buf) {
                return m
            }
        }
    }
    ret
}
/// extract the contents of content.md for editing
fn content(path: &Path) -> String {
    let mut buf = String::new();
    if let Ok(mut f) = File::open(path) {
        match f.read_to_string(&mut buf) {
            Ok(_size) => buf,
            Err(_) => String::new()
        }
    } else {
        String::new()
    }
}

pub fn write_input(state: &AppState) -> Result<(), String> {
    for project in state.website.portfolio.iter() {
        let path = state.source.join("portfolio").join(&project.meta.title);
        ensure_folder(&path)?;
        write_file(&mut project.description.clone(), path.join("content.md"))?;
        if let Ok(mut m) = toml::to_string(&project.meta) {
            write_file(&mut m, path.join("meta.toml"))?;
        }
        ensure_folder(&path.join("img"))?;
    }
    write_file(&mut state.website.about.clone(), state.source.join("about.md"))?;
    Ok(())
}

pub fn ensure_folder(path: &PathBuf) -> Result<(), String> {
    if path.exists() {
        return Ok(())
    }
    let db = DirBuilder::new();
    db.create(path).map_err(|e| format!("{:?}", e))
}

pub fn write_file(content: &mut str, path: PathBuf) -> Result<(), String> {
    match File::create(&path) {
        Ok(mut f) => {
            match f.write_all(content.as_bytes()) {
                Ok(_size) => Ok(()),
                Err(e) => Err(format!("{:?}", e))
            }
        },
        Err(e) => Err(format!("{:?}", e))
    }
}

use std::collections::HashSet;
/// Ensure that all of the top level files and folders are
/// included in the source dir
pub fn ensure_dir_defaults(source: &PathBuf) {
    let files: HashSet<PathBuf> = WalkDir::new(&source).min_depth(1).max_depth(1).into_iter().filter_map(|e| {
        match e {
            Ok(e) => {
                Some(e.path().to_path_buf())
            },
            Err(_) => None
        }
    }).collect();
    let db = DirBuilder::new();
    let fonts = source.join("fonts");
    let portfolio = source.join("portfolio");
    let templates = source.join("templates");
    let about = source.join("about.md");
    let img = source.join("me.jpg");
    if files.get(&fonts).is_none() {
        let _ = db.create(&fonts);
    }
    if files.get(&portfolio).is_none() {
        let _ = db.create(&portfolio);
    } 
    if files.get(&templates).is_none() {
        let _ = db.create(&templates);
    }
    if files.get(&about).is_none() {
        let _ = File::create(&about);
    }
    if files.get(&img).is_none() {
        let _ = File::create(&img);
    }
}

pub fn copy_file(source: &PathBuf, dest_dir: &PathBuf) -> Result<PathBuf, String> {
    if let Some(file_name) = source.file_name() {
        let dest = dest_dir.join(file_name);
        let mut i_f = File::open(source).map_err(map_e)?;
        let mut buf = vec!();
        i_f.read_to_end(&mut buf).map_err(map_e)?;
        let mut o_f = File::create(&dest).map_err(map_e)?;
        o_f.write_all(&mut buf).map_err(map_e)?;
        Ok(dest)
    } else {
        Err("Unable to get source's filename".into())
    }
}

pub fn remove(path: &PathBuf) -> Result<(), String> {
    remove_file(path).map_err(map_e)?;
    Ok(())
}

fn map_e(e: ::std::io::Error) -> String {
    format!("{:?}", e)
}