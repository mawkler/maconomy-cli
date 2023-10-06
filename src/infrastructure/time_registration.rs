use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeRegistration {
    pub meta: Meta,
    pub panes: Panes,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub container_instance_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Panes {
    pub card: Card,
    pub table: Table,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub meta: CardMeta,
    pub records: Vec<CardRecord>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardMeta {
    pub pane_name: String,
    pub row_count: i64,
    pub row_offset: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardRecord {
    pub data: CardData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardData {
    pub employeenumber: String,
    pub periodstartvar: String,
    pub periodendvar: String,
    pub employeenumbervar: String,
    pub employeenamevar: String,
    pub datevar: String,
    pub dateday1var: String,
    pub dateday2var: String,
    pub dateday3var: String,
    pub dateday4var: String,
    pub dateday5var: String,
    pub dateday6var: String,
    pub dateday7var: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    pub meta: TableMeta,
    pub records: Vec<TableRecord>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableMeta {
    pub pane_name: String,
    pub row_count: i64,
    pub row_offset: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableRecord {
    pub data: TableData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableData {
    pub linenumber: i64,
    pub jobnumber: String,
    pub numberday1: i64,
    pub numberday2: i64,
    pub numberday3: i64,
    pub numberday4: i64,
    pub numberday5: i64,
    pub numberday6: i64,
    pub numberday7: i64,
    pub jobnamevar: String,
    pub tasktextvar: String,
}
