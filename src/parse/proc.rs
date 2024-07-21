use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::proc::ProcDesc;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DdlsToml {
    pub proc: BTreeMap<String, ProcToml>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProcToml {
    pub setup: String,
    pub prepare: String,
    pub run: String,
    pub update: String,
}

impl ProcToml {
    pub fn to_desc(self, name: impl Into<String>) -> ProcDesc {
        let name = name.into();
        let dir = Path::new("proc").join(&name);
        ProcDesc {
            name,
            dir,
            setup: self.setup,
            prepare: self.prepare,
            run: self.run,
            update: self.update,
        }
    }

    pub fn from_desc(desc: ProcDesc) -> ProcToml {
        ProcToml {
            setup: desc.setup,
            prepare: desc.prepare,
            run: desc.run,
            update: desc.update,
        }
    }
}
