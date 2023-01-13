use crate::data::DataValue;

pub(crate) fn to_boolean(data: &DataValue) -> Vec<bool> {
    match data {
        DataValue::Empty => vec![],
        DataValue::Boolean(b) => b.to_vec(),
        DataValue::Number(n) => { n.iter().map(|v| *v!=0.).collect() },
        DataValue::String(s) => { s.iter().map(|v| v!="").collect() }
    }
}

pub(crate) fn to_number(data: &DataValue) -> Vec<f64> {
    match data {
        DataValue::Empty => vec![],
        DataValue::Boolean(b) => { b.iter().map(|b| if *b {1.} else {0.}).collect() },
        DataValue::Number(n) => { n.to_vec() },
        DataValue::String(s) => { s.iter().map(|v| v.parse::<f64>().unwrap_or(0.)).collect() }
    }
}

pub(crate) fn to_string(data: &DataValue) -> Vec<String> {
    match data {
        DataValue::Empty => vec![],
        DataValue::Boolean(b) => { b.iter().map(|b| (if *b {"true"} else {"false"}).to_string()).collect() },
        DataValue::Number(n) => { n.iter().map(|n| n.to_string()).collect() },
        DataValue::String(s) => { s.to_vec() }
    }
}
