#[derive(Debug)]
pub enum Cardinality {
    One,
    Many,
    Exec,
    Scalar,
}

#[derive(Debug)]
pub struct Query {
    // Parsed metadata
    pub name: String,
    pub cardinality: Cardinality,
    pub sql: Vec<String>,
    pub params: Option<Vec<ParamSpec>>,
    pub returns: Option<Vec<String>>,

    // Analyzer-populated metadata
    pub columns: Vec<ColumnInfo>,

    pub transformed_sql: String,
}

impl Query {
    pub fn sql_text(&self) -> String {
        self.sql.join("\n")
    }
}

#[derive(Debug, Clone)]
pub struct ParamSpec {
    pub name: String,
    pub rust_type: String,
}

#[derive(Debug)]
pub struct ColumnInfo {
    pub name: String,
    pub decl: String,
    pub rust_type: String,
}
