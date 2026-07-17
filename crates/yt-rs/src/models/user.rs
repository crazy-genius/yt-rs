use serde::{Deserialize, Serialize};

// short model demo only
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub login: String,
}
