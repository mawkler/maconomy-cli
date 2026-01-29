use std::collections::HashMap;
use serde::Deserialize;
use serde::Serialize;

// This file was (semi) auto-generated based on the JSON response from Maconomy. I just improved
// the struct names.
//
// I use the name "time registration" for maconomy's data model, and "time sheet" for this CLI's
// domain.

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
    #[serde(default)]
    pub links: HashMap<String,Link>,
    pub records: Vec<CardRecord>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub rel: String,
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardMeta {
    pub pane_name: String,
    pub row_count: u32,
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
    pub weeknumbervar: u8,
    pub partvar: String,
    pub fixednumberday1var: f32,
    pub fixednumberday2var: f32,
    pub fixednumberday3var: f32,
    pub fixednumberday4var: f32,
    pub fixednumberday5var: f32,
    pub fixednumberday6var: f32,
    pub fixednumberday7var: f32,
    pub totalnumberday1var: f32,
    pub totalnumberday2var: f32,
    pub totalnumberday3var: f32,
    pub totalnumberday4var: f32,
    pub totalnumberday5var: f32,
    pub totalnumberday6var: f32,
    pub totalnumberday7var: f32,
    pub regulartimeday1var: f32,
    pub regulartimeday2var: f32,
    pub regulartimeday3var: f32,
    pub regulartimeday4var: f32,
    pub regulartimeday5var: f32,
    pub regulartimeday6var: f32,
    pub regulartimeday7var: f32,
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
    pub row_count: u32,
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
    pub numberday1: f32,
    pub numberday2: f32,
    pub numberday3: f32,
    pub numberday4: f32,
    pub numberday5: f32,
    pub numberday6: f32,
    pub numberday7: f32,
    pub entrytext: String,
    pub taskname: String,
    #[serde(default)]
    pub approvalstatus: String,
    pub instancekey: String,
    pub timeregistrationunit: String,
    pub jobnamevar: String,
    pub tasktextvar: String,
}
