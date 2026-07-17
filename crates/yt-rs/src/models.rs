use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{Display, Formatter};

pub mod user;

pub struct FieldsQuery(pub(crate) Vec<String>);
impl From<Vec<String>> for FieldsQuery {
    fn from(value: Vec<String>) -> Self {
        FieldsQuery(value)
    }
}
impl Serialize for FieldsQuery {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (("fields", self.0.join(",")),).serialize(serializer)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiError {
    pub error: String,
    pub error_description: String,
}
impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Youtrack api error: {} {}", self.error, self.error_description)
    }
}
