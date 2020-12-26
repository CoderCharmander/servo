use crate::config::ServerVersion;
use log::{self, info};
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use std::io;

#[derive(Deserialize, Debug)]
pub struct ProjectVersionList {
    pub project: String,
    pub versions: Vec<String>,
}

#[derive(Deserialize)]
pub struct PatchList {
    latest: u32,
    // all: Vec<u32>,
}

#[derive(Deserialize)]
struct PatchListResponse {
    builds: PatchList,
}

impl ProjectVersionList {
    pub async fn fetch(project: &str) -> Result<ProjectVersionList, Box<dyn std::error::Error>> {
        let url = "https://papermc.io/api/v1/".to_owned() + project;
        let resp = ureq::get(&url).call().into_json_deserialize::<ProjectVersionList>()?;
        Ok(resp)
    }

    pub fn fetch_patches(
        version: (u32, u32, u32),
    ) -> Result<PatchList, Box<dyn std::error::Error>> {
        let url = format!(
            "https://papermc.io/api/v1/paper/{}.{}.{}",
            version.0, version.1, version.2
        );
        let resp = ureq::get(&url).call().into_json_deserialize::<PatchListResponse>()?;
        Ok(resp.builds)
    }

    pub fn download<T: io::Write>(
        version: &ServerVersion,
        stream: &mut T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url;
        if let Some(patch) = version.patch {
            url = format!(
                "https://papermc.io/api/v1/paper/{}.{}.{}/{}/download",
                version.minecraft.0, version.minecraft.1, version.minecraft.2, patch
            );
        } else {
            url = format!(
                "https://papermc.io/api/v1/paper/{}.{}.{}/{}/download",
                version.minecraft.0,
                version.minecraft.1,
                version.minecraft.2,
                Self::fetch_patches(version.minecraft)?.latest
            );
        }
        info!("Downloading {}", url);
        let mut resp = ureq::get(&url).call().into_reader();
        io::copy(&mut resp, stream)?;
        Ok(())
    }
}

impl Display for ProjectVersionList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.project, self.versions)
    }
}
