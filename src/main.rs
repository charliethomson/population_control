/*
from collections import defaultdict
import csv
from typing import Iterator
from pprint import pformat

def csv_to_dict(filename: str):
    with open(filename, newline='') as csvfile:
        reader = csv.reader(csvfile)
        headers = reader.__next__()
        d = defaultdict(list)
        for line in reader:
            for idx, value in enumerate(line):
                d[headers[idx]].append(value)
    return dict(d)

if __name__ == '__main__':
    d = csv_to_dict('pop_data.csv')
*/
use {
    csv::{
        Reader,
        DeserializeRecordsIter,
        Writer,
    },
    serde::{
        Deserialize, Serialize,
    }, 
};

const NUM_INIT_REPRESENTATIVES: u16 = 3;


#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
struct CensusState {
    #[serde(rename = "STATE")]
    StateNo: u8,
    #[serde(rename = "NAME")]
    StateName: String,
    #[serde(rename = "CENSUS2010POP")]
    Population2010: u128,
    #[serde(rename = "POPESTIMATE2018")]
    Population2018: u128,
}

#[derive(Debug, Clone, Serialize)]
struct RepresentativeCount {
    #[serde(rename = "state")]
    StateName: String,
    Representatives: u16,
    Ratio: u128,
}

fn decode_csv() -> Vec<CensusState> {
    let mut reader = Reader::from_path("pop_data.csv").unwrap();
    let mut iter: DeserializeRecordsIter<_, CensusState> = reader.deserialize();
    let mut output = Vec::with_capacity(64);
    while let Some(record) = iter.next() {
        output.push(record.unwrap());
    }
    output
}

fn get_representatives(census_data: &Vec<CensusState>) -> (Vec<RepresentativeCount>, Vec<RepresentativeCount>) {
    let mut counts_2010 = Vec::with_capacity(50);
    let mut counts_2018 = Vec::with_capacity(50);

    //: TODO:

    // Filter states
    let states = census_data
                    .clone()
                    .into_iter()
                    .filter(
                        |state|
                            // 0 -> Regions, 72 -> Puerto Rico, 11 -> Washington D.C
                            ![0, 72, 11].contains(&state.StateNo)
                    )
                    .collect::<Vec<CensusState>>();
    //


    // initialise representitives
    let mut total_representatives: u16 = 50 * NUM_INIT_REPRESENTATIVES;
    for state in states.clone().into_iter() {
        counts_2010.push(RepresentativeCount {
            StateName: state.StateName.clone(),
            Representatives: NUM_INIT_REPRESENTATIVES,
            Ratio: state.Population2010.checked_div(NUM_INIT_REPRESENTATIVES.into()).unwrap(),
        });
        counts_2018.push(RepresentativeCount {
            StateName: state.StateName.clone(),
            Representatives: NUM_INIT_REPRESENTATIVES,
            Ratio: state.Population2018.checked_div(NUM_INIT_REPRESENTATIVES.into()).unwrap(),
        });
    }
    //


    // 2010
    while total_representatives != 435 {
        let (max_idx, mut max) = counts_2010.iter().enumerate().max_by(|(_, a), (_, b)| a.Ratio.cmp(&b.Ratio)).map(|(i, a)| (i, a.clone())).unwrap();
        let population = states.clone().iter().filter(|state| state.StateName == max.StateName).map(|state| state.Population2010).next().unwrap();
        eprintln!("max:\n{:#?} cur_rep: {} new_rep: {} population: {}", max, max.Representatives, max.Representatives+1, population);
        max.Representatives += 1;
        let tr = max.Ratio;
        max.Ratio = population.checked_div(max.Representatives.into()).unwrap();
        eprintln!("old_ratio: {} new_ratio: {}", tr, max.Ratio);
        counts_2010.remove(max_idx);
        counts_2010.push(max);

        total_representatives += 1;
    }


    //

    (counts_2010, counts_2018)
}


fn main() {

    let census_data = decode_csv();    
    eprintln!("{:#?}", get_representatives(&census_data).0);

    let mut writer = Writer::from_path("dump.csv").unwrap();
    let mut representatives = get_representatives(&census_data).0;
    representatives.sort_by(|a,b| a.Ratio.cmp(&b.Ratio));
    for c in representatives {
        writer.serialize(c).unwrap();
    }
}