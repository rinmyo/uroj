pub mod station;

#[cfg(test)]
mod tests {
    use super::station::*;
    use futures::stream::StreamExt;
    use wither::mongodb::Client;
    use wither::{prelude::*, Result};

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }


}
