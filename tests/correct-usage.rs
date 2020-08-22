use const_table::const_table;
use std::convert::TryFrom;

/// Identifiers for all recognized species.
#[const_table]
pub enum SpeciesID {
    /// Bundle of all relevant attributes of a species.
    #[derive(Debug)]
    SpeciesInfo {
        pub sound: &'static str,
        pub legs: u64,
    },
    Cat = SpeciesInfo {
        sound: "Meow!",
        legs: 4,
    },
    Dog = SpeciesInfo {
        sound: "Woof!",
        legs: 4,
    },
    Human = SpeciesInfo {
        sound: "Hello, World",
        legs: 2,
    },
}

fn main() {
    use SpeciesID::*;

    assert_eq!(std::mem::size_of::<SpeciesID>(), std::mem::size_of::<u32>());
    assert_eq!(
        std::mem::size_of::<SpeciesID>(),
        std::mem::size_of::<Option<SpeciesID>>()
    );
    assert_eq!(
        SpeciesID::iter().collect::<Vec<SpeciesID>>(),
        [Cat, Dog, Human]
    );

    assert_eq!(Ok(Cat), SpeciesID::try_from(0));
    assert_eq!(Ok(Dog), SpeciesID::try_from(1));
    assert_eq!(Ok(Human), SpeciesID::try_from(2));
    assert_eq!(Err(3), SpeciesID::try_from(3));

    assert_eq!(format!("{}", Human.sound), "Hello, World");
    assert_eq!(format!("{:?}", *Cat), "SpeciesInfo { sound: \"Meow!\", legs: 4 }");
}
