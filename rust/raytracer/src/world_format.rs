use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use crate::hit::World;

#[derive(Deserialize, Serialize)]
pub struct WorldInfo {
    version: String,
    world: World,
}

impl WorldInfo {
    pub fn new(world: World) -> Self {
        WorldInfo {
            version: env!("CARGO_PKG_VERSION").into(),
            world,
        }
    }

    pub fn validate(self) -> World {
        let version_requirement =
            VersionReq::parse("0.1.*").expect("Unable to parse baked-in version requirement.");
        if version_requirement.matches(&Version::parse(self.version.as_str()).expect(format!(
            "The world file reports a version of {}, which is not a valid version.",
            self.version
        ).as_str())) {
            self.world
        } else {
            panic!("The world file reports a version of {}, but this version of the program requires a file with a version that matches {}.", self.version, version_requirement);
        }
    }
}
