#[derive(Debug)]
pub enum Cardinality {
    One,
    Many,
    Exec,
    Scalar,
}

#[derive(Debug)]
pub struct ParsedQuery {
    pub name: String,
    pub cardinality: Cardinality,
    pub sql: Vec<String>,
    pub params: Option<Vec<ParsedParamInfo>>,
    pub returns: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct ParsedParamInfo {
    pub name: String,
    pub rust_type: String,
}

pub struct QueryInfo {
    pub columns: Vec<ColumnInfo>,
    pub params: Vec<String>,
}

pub struct ColumnInfo {
    pub name: String,
    pub decl: String,
    pub rust_type: String,
}