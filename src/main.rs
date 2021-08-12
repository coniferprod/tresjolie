use std::env;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use serde_json::json;
use reqwest::blocking;
use indicatif::ProgressBar;

const JOURNEYS_API: &str = "http://data.itsfactory.fi/journeys/api/1";
const ENDPOINT_STOP_POINTS: &str = "stop-points";
const ENDPOINT_JOURNEYS: &str = "journeys";
const ENDPOINT_LINES: &str = "lines";

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
struct StopPoint {
    url: String,
    location: String,
    name: String,
    short_name: String,
    tariff_zone: String,
    municipality: Municipality,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Municipality {
    url: String,
    short_name: String,
    name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
struct JourneysResponse {
    status: String,
    data: serde_json::Value,
    body: serde_json::Value,
}

#[derive(Serialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
struct Stop {
    code: String,
    name: String,
    latitude: f64,
    longitude: f64,
    lines: Vec<String>,
    direction: String,
    municipality: String,
    zone: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Line {
    url: String,
    name: String,
    description: String,
}

fn get_lines_for_stop(stop_code: &str) -> Vec<Line> {
    let mut url = [[JOURNEYS_API, ENDPOINT_LINES].join("/"), "?stopPointId=".to_string(), stop_code.to_string()].concat();
    let res = reqwest::blocking::get(url).unwrap();
    let json: JourneysResponse = res.json().unwrap();
    let lines: Vec<Line> = serde_json::from_value(json.body).unwrap();
    lines
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let url = [JOURNEYS_API, ENDPOINT_STOP_POINTS].join("/");
    let res = reqwest::blocking::get(url)?;
    let json: JourneysResponse = res.json()?;
    let stop_points: Vec<StopPoint> = serde_json::from_value(json.body).unwrap();

    //println!("Got {} stop points from API", stop_points.len());

    let mut stops: Vec<Stop> = Vec::<Stop>::new();

    let pb = ProgressBar::new(stop_points.len() as u64);

    for sp in stop_points.iter() {
        let lines = get_lines_for_stop(&sp.short_name);

        //println!("Got {} lines for stop {}", lines.len(), sp.short_name);

        let stop_lines = lines.iter().map(|line| line.name.to_string()).collect();

        let coordinates: Vec<&str> = sp.location.split(",").collect();
        let stop = Stop {
            name: sp.name.to_string(),
            code: sp.short_name.to_string(),
            latitude: coordinates[0].parse().unwrap(),
            longitude: coordinates[1].parse().unwrap(),
            direction: "".to_string(),
            lines: stop_lines,
            municipality: sp.municipality.short_name.to_string(),
            zone: sp.tariff_zone.to_string(),
        };

        stops.push(stop);

        pb.inc(1);
    }

    pb.finish_with_message("Done");

    let j = serde_json::to_string_pretty(&stops)?;
    println!("{}", j);

    Ok(())
}
