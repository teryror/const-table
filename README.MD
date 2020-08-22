`#[const_table]`
==============

This crate provides an attribute macro to associate struct-type constants with enum variants.

```toml
[dependencies]
const-table = "0.1"
```

## Syntax

Place `#[const_table]` on an enum with at least two variants, where

* the first has named fields and defines the type of the associated constants, and
* all following have discriminant expressions of that type:

```rust
#[const_table]
pub enum Planet {
    PlanetInfo {
        pub mass: f32,
        pub radius: f32,
    },

    Mercury = PlanetInfo { mass: 3.303e+23, radius: 2.4397e6 },
    Venus = PlanetInfo { mass: 4.869e+24, radius: 6.0518e6 },
    Earth = PlanetInfo { mass: 5.976e+24, radius: 6.37814e6 },
    Mars = PlanetInfo { mass: 6.421e+23, radius: 3.3972e6 },
    Jupiter = PlanetInfo { mass: 1.9e+27, radius: 7.1492e7 },
    Saturn = PlanetInfo { mass: 5.688e+26, radius: 6.0268e7 },
    Uranus = PlanetInfo { mass: 8.686e+25, radius: 2.5559e7 },
    Neptune = PlanetInfo { mass: 1.024e+26, radius: 2.4746e7 },
}
```

This expands to the following:

```rust
#[repr(u32)]
#[derive(core::marker::Copy, core::clone::Clone, core::fmt::Debug, core::hash::Hash, core::cmp::PartialEq, core::cmp::Eq)]
pub enum Planet {
    Mercury,
    Venus,
    Earth,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
}

pub struct PlanetInfo {
    pub mass: f32,
    pub radius: f32,
}

impl Planet {
    const COUNT: usize = 8;
    pub fn iter() -> impl core::iter::DoubleEndedIterator<Item = Self> {
        // transmuting here is fine because... (see try_from)
        (0..Self::COUNT).map(|i| unsafe { core::mem::transmute(i as u32) })
    }
}

impl core::ops::Deref for Planet {
    type Target = PlanetInfo;
    fn deref(&self) -> &Self::Target {
        use Planet::*;
        const TABLE: [PlanetInfo; 8] = [
            PlanetInfo { mass: 3.303e+23, radius: 2.4397e6 },
            PlanetInfo { mass: 4.869e+24, radius: 6.0518e6 },
            PlanetInfo { mass: 5.976e+24, radius: 6.37814e6 },
            PlanetInfo { mass: 6.421e+23, radius: 3.3972e6 },
            PlanetInfo { mass: 1.9e+27, radius: 7.1492e7 },
            PlanetInfo { mass: 5.688e+26, radius: 6.0268e7 },
            PlanetInfo { mass: 8.686e+25, radius: 2.5559e7 },
            PlanetInfo { mass: 1.024e+26, radius: 2.4746e7 },
        ];

        &TABLE[*self as usize]
    }
}

impl core::convert::TryFrom<u32> for Planet {
    type Error = u32;
    fn try_from(i: u32) -> Result<Self, Self::Error> {
        if (i as usize) < Self::COUNT {
            // transmuting here is fine because all values in range are valid, since
            // discriminants are assigned linearly starting at 0.
            Ok(unsafe { core::mem::transmute(i) })
        } else {
            Err(i)
        }
    }
}
```

Note the automatically inserted `repr` and `derive` attributes. You may place a different `repr` attribute as normal,
although only `u8`, `u16`, `u32` and `u64` are supported; an implementation of `TryFrom<T>` is provided, where `T` is
the chosen `repr` type. You may also `derive` additional traits on the enum.

Any attributes placed on the first variant will be placed on the corresponding struct in the expanded code.

Also, note that the macro places the discriminant expressions inside a scope that imports all variants of your enum.
This makes it convenient to make the values refer to each other, e.g. in a graph-like structure. 

Because the macro implements `Deref` for your enum, you can access fields of the target type like `Planet::Earth.mass`.

Finally, `Planet::iter()` gives a `DoubleEndedIterator` over all variants in declaration order, and `Planet::COUNT` is
the total number of variants.

## License

Licensed under either of <a href="LICENSE-APACHE">Apache License, Version 2.0</a> or
<a href="LICENSE-MIT">MIT license</a> at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you,
as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
