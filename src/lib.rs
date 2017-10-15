extern crate bincode;
extern crate gltf;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub fn test() -> u64 {
    5_u64
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
        assert_eq!(test(), 5_u64);
    }
}
