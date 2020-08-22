use const_table::const_table;

#[const_table]
#[derive(Debug)]
enum SpeciesID {
    SpeciesInfo { pub sound: &'static str },
    Human = SpeciesInfo { sound: "Hello World" }
}

fn main() {}