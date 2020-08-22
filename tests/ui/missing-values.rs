use const_table::const_table;

#[const_table]
enum Foo {
    Bar { bazz: i32 }
}

fn main() {}