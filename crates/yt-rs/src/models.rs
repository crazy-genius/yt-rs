use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{Display, Formatter};

mod generated;
pub use generated::*;

pub struct FieldsQuery(pub(crate) Vec<String>);
impl From<Vec<String>> for FieldsQuery {
    fn from(value: Vec<String>) -> Self {
        FieldsQuery(value)
    }
}
impl FieldsQuery {
    /// Joined field list with the `$type` tag injected exactly once.
    pub(crate) fn into_field_value(self) -> String {
        let mut joined = self.0.join(",");
        if !self.0.iter().any(|f| f == "$type") {
            joined = if joined.is_empty() { "$type".to_owned() } else { format!("$type,{joined}") };
        }
        joined
    }
}
impl Serialize for FieldsQuery {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // The generated polymorphic enums dispatch on `$type`, so it must always
        // be requested; nested selections are the caller's responsibility.
        let value = FieldsQuery(self.0.clone()).into_field_value();
        (("fields", value),).serialize(serializer)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn fields_string(q: &FieldsQuery) -> String {
        // FieldsQuery serializes as (("fields", <joined>),)
        let v = serde_json::to_value(q).unwrap();
        v[0][1].as_str().unwrap().to_owned()
    }

    #[test]
    fn fields_query_injects_type_tag() {
        let q: FieldsQuery = vec!["login".to_string(), "id".into()].into();
        assert_eq!(fields_string(&q), "$type,login,id");
    }

    #[test]
    fn fields_query_does_not_duplicate_type_tag() {
        let q: FieldsQuery = vec!["$type".to_string(), "login".into()].into();
        assert_eq!(fields_string(&q), "$type,login");
    }

    #[test]
    fn empty_fields_query_still_requests_type_tag() {
        let q: FieldsQuery = Vec::<String>::new().into();
        assert_eq!(fields_string(&q), "$type");
    }

    #[test]
    fn nested_selections_pass_through() {
        let q: FieldsQuery = vec!["customFields($type,id,value($type,name))".to_string()].into();
        assert_eq!(fields_string(&q), "$type,customFields($type,id,value($type,name))");
    }
}
