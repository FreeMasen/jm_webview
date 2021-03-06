use std::path::{PathBuf, Path};

use walkdir::{WalkDir};

use fonts::Fonts;
use meta::Meta;
use project::Project;
use error::{StateError, StateResult};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Website {
    pub title: String,
    pub portfolio: Vec<Project>,
    pub about: String,
    pub image: PathBuf,
    pub fonts: Fonts,
    pub accent_color: Color,
}

impl Website {
    pub fn new(title: &String) -> Website {
        Website {
            title: title.clone(),
            ..Website::default()
        }
    }
    pub fn add_project(&mut self, path_root: &PathBuf, name: String) {
        let new_project = Project {
            id: self.portfolio.len() as u32,
            path: path_root.join("portfolio").join(&name),
            meta: Meta {
                title: name,
                ..Meta::default()
            },
            ..Project::default()
        };
        self.portfolio.push(new_project);
    }

    pub fn update_project(&mut self, project: Project) {
        println!("update_project with id {}", &project.id);
        match self.get_project_idx(project.id) {
            Some(idx) => {
                println!("found project {}", &self.portfolio[idx].meta.title);
                self.portfolio[idx] = project
            },
            None => println!("Unable to find project with matching id"),
        }
    }

    fn get_project_idx(&self, id: u32) -> Option<usize> {
        self.portfolio.iter().position(|p| p.id == id)
    }

    pub fn update_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn update_projects_from_source(&mut self, path: &Path) {
        let mut tmp_portfolio: Vec<Project> = vec!();
        for entry in WalkDir::new(path).min_depth(1).max_depth(1) {
            if let Ok(entry) = entry {
                match self.portfolio.iter().position(|p| p.path() == entry.path().to_path_buf()) {
                    Some(idx) => {
                        let mut p = self.portfolio[idx].clone();
                        p.id = tmp_portfolio.len() as u32;
                        p.path = entry.path().to_path_buf();
                        p.update_from_source();
                        tmp_portfolio.push(p);

                    },
                    None => {
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
    pub fn delete_project(&mut self, project: &Project) -> StateResult {
            match project.delete_files() {
                Ok(()) => {
                    self.portfolio = self.portfolio.clone().into_iter().filter(|p| p.id != project.id).collect();
                    Ok("Successfully deleted project".into())
                },
                Err(e) => Err(StateError::new(format!("{:?}", e))),
            }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: f32,
}