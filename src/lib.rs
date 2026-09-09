// common/src/lib.rs

pub mod model {

    pub mod grid;
    pub mod item {
        pub mod durability_item;
        pub mod simple_item;
    }
    pub mod status;
}

pub mod utility {
    pub mod console;
    pub mod io;
    pub mod random;
    pub mod time;
}

pub mod examples {
    pub mod like_ken {
        pub mod status {
            pub mod character;
        }
    }
    pub mod demo;
}
