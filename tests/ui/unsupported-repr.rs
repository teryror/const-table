use const_table::const_table;

const fn p(mass: f32, radius: f32) -> PlanetInfo {
    PlanetInfo { mass, radius }
}

#[repr(i32)]
#[const_table]
enum Planet {
    PlanetInfo { mass: f32, radius: f32 },

    Mercury = p(3.303e+23, 2.4397e6),
    Venus = p(4.869e+24, 6.0518e6),
    Earth = p(5.976e+24, 6.37814e6),
    Mars = p(6.421e+23, 3.3972e6),
    Jupiter = p(1.9e+27, 7.1492e7),
    Saturn = p(5.688e+26, 6.0268e7),
    Uranus = p(8.686e+25, 2.5559e7),
    Neptune = p(1.024e+26, 2.4746e7),
}

fn main() {}
