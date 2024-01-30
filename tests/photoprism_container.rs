use std::collections::HashMap;

use testcontainers::core::WaitFor;
use testcontainers::Image;

pub struct PhotoPrismContainer {
    env_vars: HashMap<String, String>,
}

impl Default for PhotoPrismContainer {
    fn default() -> Self {
        Self {
            env_vars: HashMap::from([
                ("PHOTOPRISM_UPLOAD_NSFW".to_owned(), "true".to_owned()),
                ("PHOTOPRISM_ADMIN_PASSWORD".to_owned(), "insecure".to_owned()),
                ("PHOTOPRISM_DEBUG".to_owned(), "true".to_owned())
            ])
        }
    }
}

impl Image for PhotoPrismContainer {
    type Args = ();

    fn name(&self) -> String {
        "photoprism/photoprism".to_string()
    }

    fn tag(&self) -> String {
        "latest".to_string()
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr("server: listening on")]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item=(&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}