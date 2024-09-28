use serde::Deserialize;
use serde::Serialize;

// This file was (semi) auto-generated based on the JSON response from Maconomy. I just improved the struct names.

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
    pub employeenamevar: String,
    pub datevar: String,
    pub weeknumbervar: i64,
    pub fixednumberday1var: i64,
    pub fixednumberday2var: i64,
    pub fixednumberday3var: i64,
    pub fixednumberday4var: i64,
    pub fixednumberday5var: i64,
    pub fixednumberday6var: i64,
    pub fixednumberday7var: i64,
    pub totalnumberday1var: i64,
    pub totalnumberday2var: i64,
    pub totalnumberday3var: i64,
    pub totalnumberday4var: i64,
    pub totalnumberday5var: i64,
    pub totalnumberday6var: i64,
    pub totalnumberday7var: i64,
    pub regulartimeday1var: i64,
    pub regulartimeday2var: i64,
    pub regulartimeday3var: i64,
    pub regulartimeday4var: i64,
    pub regulartimeday5var: i64,
    pub regulartimeday6var: i64,
    pub regulartimeday7var: i64,
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
    pub jobnumber: String,
    pub numberday1: i64,
    pub numberday2: i64,
    pub numberday3: i64,
    pub numberday4: i64,
    pub numberday5: i64,
    pub numberday6: i64,
    pub numberday7: i64,
    pub entrytext: String,
    pub taskname: String,
    pub instancekey: String,
    pub timeregistrationunit: String,
    pub jobnamevar: String,
    pub tasktextvar: String,
}
