
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

// House representatives + senate (minimum)
const NUM_INIT_REPRESENTATIVES: u16 = 3;
const DIV_BY_ZERO_ERROR: &'static str = "Error! Attempt to divide by zero. If NUM_INIT_REPRESENTATIVES\
is set to 0, please fix that :)";
const POPULATION_CSV_PATH: &'static str = "csv/pop_data.csv";


#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
struct CensusState {
    #[serde(rename = "STATE")]
    state_no: u8,
    #[serde(rename = "NAME")]
    state_name: String,
    #[serde(rename = "CENSUS2010POP")]
    population_2010: u128,
    #[serde(rename = "POPESTIMATE2018")]
    population_2018: u128,
}

#[derive(Debug, Clone, Serialize)]
struct RepresentativeCount {
    #[serde(rename = "state")]
    state_name: String,
    representatives: u16,
    ratio: u128,
}

fn decode_csv() -> Vec<CensusState> {
    let mut reader = Reader::from_path(POPULATION_CSV_PATH).unwrap();
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

    //: TODO: Reformat

    // Filter states
    let states = census_data
                    .clone()
                    .into_iter()
                    .filter(
                        |state|
                            // 0 -> Regions, 72 -> Puerto Rico, 11 -> Washington D.C
                            ![0, 72, 11].contains(&state.state_no)
                    )
                    .collect::<Vec<CensusState>>();
    //


    // initialise representatives
    let mut total_representatives: u16 = 50 * NUM_INIT_REPRESENTATIVES;
    for state in states.clone().into_iter() {
        counts_2010.push(RepresentativeCount {
            state_name: state.state_name.clone(),
            representatives: NUM_INIT_REPRESENTATIVES,
            ratio: state.population_2010.checked_div(NUM_INIT_REPRESENTATIVES.into()).unwrap(),
        });
        counts_2018.push(RepresentativeCount {
            state_name: state.state_name.clone(),
            representatives: NUM_INIT_REPRESENTATIVES,
            ratio: state.population_2018.checked_div(NUM_INIT_REPRESENTATIVES.into()).unwrap(),
        });
    }
    //


    // 2010
    while total_representatives != 435 {
        // Get the index and a clone of the RepresentativeCount with the highest ratio
        let (max_idx, mut max) = counts_2010
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.ratio.cmp(&b.ratio)
            })
            .map(|(i, r)| {
                (i, r.clone())
            })
            .unwrap();
        // Get the total population of the state represented by `max`
        // This will do for now, but when I properly reformat, I'll remove the need for this at all
        let population = states
            .clone()
            .iter()
            .filter(|state| state.state_name == max.state_name)
            .map(|state| state.population_2010)
            .next()
            .unwrap();

        // Modify the clone of the state with the highest ratio
        max.representatives += 1;
        max.ratio = population.checked_div(max.representatives.into()).expect(DIV_BY_ZERO_ERROR);
        // remove the old version of `max` (this is why we needed `max_idx`), and replace it with
        // the modified version
        counts_2010.remove(max_idx);
        counts_2010.push(max);

        total_representatives += 1;
    }
    //

    // 2018
    // The code for this section is almost exactly the same as the code for 2010, I'll probably make
    // it a macro or reformat
    total_representatives = 50 * NUM_INIT_REPRESENTATIVES;
    while total_representatives != 435 {
        // Get the index and a clone of the RepresentativeCount with the highest ratio
        let (max_idx, mut max) = counts_2018
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| {
                a.ratio.cmp(&b.ratio)
            })
            .map(|(i, r)| {
                (i, r.clone())
            })
            .unwrap();
        // Get the total population of the state represented by `max`
        // This will do for now, but when I properly reformat, I'll remove the need for this at all
        let population = states
            .clone()
            .iter()
            .filter(|state| state.state_name == max.state_name)
            .map(|state| state.population_2018)
            .next()
            .unwrap();

        // Modify the clone of the state with the highest ratio
        max.representatives += 1;
        max.ratio = population.checked_div(max.representatives.into()).expect(DIV_BY_ZERO_ERROR);
        // remove the old version of `max` (this is why we needed `max_idx`), and replace it with
        // the modified version
        counts_2018.remove(max_idx);
        counts_2018.push(max);

        total_representatives += 1;
    }
    //

    (counts_2010, counts_2018)
}


fn main() {

    let census_data = decode_csv();    

    let mut writer2010 = Writer::from_path("csv/dump2010.csv").unwrap();
    let mut writer2018 = Writer::from_path("csv/dump2018.csv").unwrap();

    let mut representatives = get_representatives(&census_data);

    representatives.0.sort_by(|a,b| a.ratio.cmp(&b.ratio));
    representatives.1.sort_by(|a,b| a.ratio.cmp(&b.ratio));

    for (t, e) in representatives.0.iter().zip(representatives.1.iter()) {
        writer2010.serialize(t).unwrap();
        writer2018.serialize(e).unwrap();
    }
}