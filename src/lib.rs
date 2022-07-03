pub struct GHRepo {
    owner: String,
    name: String,
}

impl GHRepo {
    fn owner(&self) -> &str {
        &self.owner
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn fullname(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }
}
