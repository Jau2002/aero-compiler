use csv::StringRecord;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct DataSet {
    pub pilotos: Vec<Row>,
    pub aeronaves: Vec<Row>,
    pub vuelos: Vec<Row>,
}

pub type Row = HashMap<String, String>;

pub fn load_data(dir: &Path) -> Result<DataSet, String> {
    Ok(DataSet {
        pilotos: load_csv(&dir.join("pilotos.csv"))?,
        aeronaves: load_csv(&dir.join("aeronaves.csv"))?,
        vuelos: load_csv(&dir.join("vuelos.csv"))?,
    })
}

fn load_csv(path: &Path) -> Result<Vec<Row>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut reader = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
    let headers = reader.headers().map_err(|e| e.to_string())?.clone();
    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|e| e.to_string())?;
        rows.push(record_to_row(&headers, &record));
    }
    Ok(rows)
}

fn record_to_row(headers: &StringRecord, record: &StringRecord) -> Row {
    let mut row = Row::new();
    for (key, value) in headers.iter().zip(record.iter()) {
        row.insert(key.to_string(), value.to_string());
    }
    row
}
