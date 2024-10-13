use serde::Serialize;

#[derive(Serialize)]
pub struct ElectionContests<'a> {
    pub(crate) election: &'a str,
    pub(crate) contests: &'a str,
}
