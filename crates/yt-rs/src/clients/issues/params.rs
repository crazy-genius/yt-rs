use crate::FieldsQuery;

#[derive(Default)]
pub struct ListParams {
    top: Option<i64>,
    skip: Option<i64>,
    fields: Option<FieldsQuery>,
}
impl ListParams {
    pub fn top(mut self, v: i64) -> Self {
        self.top = Some(v);
        self
    }
    pub fn skip(mut self, v: i64) -> Self {
        self.skip = Some(v);
        self
    }
    pub fn fields(mut self, v: FieldsQuery) -> Self {
        self.fields = Some(v);
        self
    }
    pub(crate) fn into_query(self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        if let Some(t) = self.top {
            q.push(("$top", t.to_string()));
        }
        if let Some(s) = self.skip {
            q.push(("$skip", s.to_string()));
        }
        if let Some(f) = self.fields {
            q.push(("fields", f.into_field_value()));
        }
        q
    }
}

#[derive(Default)]
pub struct IssueSearchParams {
    query: Option<String>,
    custom_fields: Vec<String>,
    top: Option<i64>,
    skip: Option<i64>,
    fields: Option<FieldsQuery>,
}
impl IssueSearchParams {
    pub fn query(mut self, v: impl Into<String>) -> Self {
        self.query = Some(v.into());
        self
    }
    pub fn custom_field(mut self, v: impl Into<String>) -> Self {
        self.custom_fields.push(v.into());
        self
    }
    pub fn top(mut self, v: i64) -> Self {
        self.top = Some(v);
        self
    }
    pub fn skip(mut self, v: i64) -> Self {
        self.skip = Some(v);
        self
    }
    pub fn fields(mut self, v: FieldsQuery) -> Self {
        self.fields = Some(v);
        self
    }
    pub(crate) fn into_query(self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        if let Some(v) = self.query {
            q.push(("query", v));
        }
        for cf in self.custom_fields {
            q.push(("customFields", cf));
        }
        if let Some(t) = self.top {
            q.push(("$top", t.to_string()));
        }
        if let Some(s) = self.skip {
            q.push(("$skip", s.to_string()));
        }
        if let Some(f) = self.fields {
            q.push(("fields", f.into_field_value()));
        }
        q
    }
}

#[derive(Default)]
pub struct MutationParams {
    mute_update_notifications: Option<bool>,
    draft_id: Option<String>,
    fields: Option<FieldsQuery>,
}
impl MutationParams {
    pub fn mute(mut self, v: bool) -> Self {
        self.mute_update_notifications = Some(v);
        self
    }
    pub fn draft_id(mut self, v: impl Into<String>) -> Self {
        self.draft_id = Some(v.into());
        self
    }
    pub fn fields(mut self, v: FieldsQuery) -> Self {
        self.fields = Some(v);
        self
    }
    pub(crate) fn into_query(self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        if let Some(m) = self.mute_update_notifications {
            q.push(("muteUpdateNotifications", m.to_string()));
        }
        if let Some(d) = self.draft_id {
            q.push(("draftId", d));
        }
        if let Some(f) = self.fields {
            q.push(("fields", f.into_field_value()));
        }
        q
    }
}

#[derive(Default)]
pub struct ActivityParams {
    categories: Vec<String>,
    reverse: Option<bool>,
    start: Option<i64>,
    end: Option<i64>,
    author: Option<String>,
    top: Option<i64>,
    skip: Option<i64>,
    fields: Option<FieldsQuery>,
}
impl ActivityParams {
    pub fn category(mut self, v: impl Into<String>) -> Self {
        self.categories.push(v.into());
        self
    }
    pub fn reverse(mut self, v: bool) -> Self {
        self.reverse = Some(v);
        self
    }
    pub fn start(mut self, v: i64) -> Self {
        self.start = Some(v);
        self
    }
    pub fn end(mut self, v: i64) -> Self {
        self.end = Some(v);
        self
    }
    pub fn author(mut self, v: impl Into<String>) -> Self {
        self.author = Some(v.into());
        self
    }
    pub fn top(mut self, v: i64) -> Self {
        self.top = Some(v);
        self
    }
    pub fn skip(mut self, v: i64) -> Self {
        self.skip = Some(v);
        self
    }
    pub fn fields(mut self, v: FieldsQuery) -> Self {
        self.fields = Some(v);
        self
    }
    pub(crate) fn into_query(self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        for c in self.categories {
            q.push(("categories", c));
        }
        if let Some(r) = self.reverse {
            q.push(("reverse", r.to_string()));
        }
        if let Some(s) = self.start {
            q.push(("start", s.to_string()));
        }
        if let Some(e) = self.end {
            q.push(("end", e.to_string()));
        }
        if let Some(a) = self.author {
            q.push(("author", a));
        }
        if let Some(t) = self.top {
            q.push(("$top", t.to_string()));
        }
        if let Some(s) = self.skip {
            q.push(("$skip", s.to_string()));
        }
        if let Some(f) = self.fields {
            q.push(("fields", f.into_field_value()));
        }
        q
    }
}

#[derive(Default)]
pub struct ActivityPageParams {
    categories: Vec<String>,
    reverse: Option<bool>,
    start: Option<i64>,
    end: Option<i64>,
    author: Option<String>,
    cursor: Option<String>,
    activity_id: Option<String>,
    fields: Option<FieldsQuery>,
}
impl ActivityPageParams {
    pub fn category(mut self, v: impl Into<String>) -> Self {
        self.categories.push(v.into());
        self
    }
    pub fn reverse(mut self, v: bool) -> Self {
        self.reverse = Some(v);
        self
    }
    pub fn start(mut self, v: i64) -> Self {
        self.start = Some(v);
        self
    }
    pub fn end(mut self, v: i64) -> Self {
        self.end = Some(v);
        self
    }
    pub fn author(mut self, v: impl Into<String>) -> Self {
        self.author = Some(v.into());
        self
    }
    pub fn cursor(mut self, v: impl Into<String>) -> Self {
        self.cursor = Some(v.into());
        self
    }
    pub fn activity_id(mut self, v: impl Into<String>) -> Self {
        self.activity_id = Some(v.into());
        self
    }
    pub fn fields(mut self, v: FieldsQuery) -> Self {
        self.fields = Some(v);
        self
    }
    pub(crate) fn into_query(self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        for c in self.categories {
            q.push(("categories", c));
        }
        if let Some(r) = self.reverse {
            q.push(("reverse", r.to_string()));
        }
        if let Some(s) = self.start {
            q.push(("start", s.to_string()));
        }
        if let Some(e) = self.end {
            q.push(("end", e.to_string()));
        }
        if let Some(a) = self.author {
            q.push(("author", a));
        }
        if let Some(c) = self.cursor {
            q.push(("cursor", c));
        }
        if let Some(a) = self.activity_id {
            q.push(("activityId", a));
        }
        if let Some(f) = self.fields {
            q.push(("fields", f.into_field_value()));
        }
        q
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FieldsQuery;

    #[test]
    fn list_params_emits_only_set_fields() {
        let q = ListParams::default().top(10).skip(5).into_query();
        assert_eq!(q, vec![("$top", "10".to_string()), ("$skip", "5".to_string())]);
    }

    #[test]
    fn list_params_injects_type_into_fields() {
        let q = ListParams::default()
            .fields(FieldsQuery::from(vec!["id".to_string(), "summary".into()]))
            .into_query();
        assert_eq!(q, vec![("fields", "$type,id,summary".to_string())]);
    }

    #[test]
    fn search_params_repeats_custom_fields() {
        let q = IssueSearchParams::default()
            .query("state: Open")
            .custom_field("Priority")
            .custom_field("Type")
            .top(50)
            .into_query();
        assert_eq!(
            q,
            vec![
                ("query", "state: Open".to_string()),
                ("customFields", "Priority".to_string()),
                ("customFields", "Type".to_string()),
                ("$top", "50".to_string()),
            ]
        );
    }

    #[test]
    fn mutation_params_emits_bool_and_draft() {
        let q = MutationParams::default().mute(true).draft_id("draft-1").into_query();
        assert_eq!(
            q,
            vec![
                ("muteUpdateNotifications", "true".to_string()),
                ("draftId", "draft-1".to_string()),
            ]
        );
    }

    #[test]
    fn mutation_params_injects_type_into_fields() {
        let q = MutationParams::default()
            .mute(false)
            .fields(FieldsQuery::from(vec!["id".to_string(), "summary".into()]))
            .into_query();
        assert_eq!(
            q,
            vec![
                ("muteUpdateNotifications", "false".to_string()),
                ("fields", "$type,id,summary".to_string()),
            ]
        );
    }

    #[test]
    fn activity_params_repeats_categories() {
        let q = ActivityParams::default()
            .category("CommentsCategory")
            .category("IssueCreatedCategory")
            .reverse(true)
            .into_query();
        assert_eq!(
            q,
            vec![
                ("categories", "CommentsCategory".to_string()),
                ("categories", "IssueCreatedCategory".to_string()),
                ("reverse", "true".to_string()),
            ]
        );
    }

    #[test]
    fn activity_page_params_emits_cursor() {
        let q = ActivityPageParams::default().cursor("abc").activity_id("2-3").into_query();
        assert_eq!(q, vec![("cursor", "abc".to_string()), ("activityId", "2-3".to_string()),]);
    }

    #[test]
    fn search_params_full_call_shape() {
        let q = IssueSearchParams::default()
            .query("for: me #Unresolved")
            .top(20)
            .skip(40)
            .fields(FieldsQuery::from(vec!["id".to_string(), "summary".into()]))
            .into_query();
        assert_eq!(
            q,
            vec![
                ("query", "for: me #Unresolved".to_string()),
                ("$top", "20".to_string()),
                ("$skip", "40".to_string()),
                ("fields", "$type,id,summary".to_string()),
            ]
        );
    }
}
