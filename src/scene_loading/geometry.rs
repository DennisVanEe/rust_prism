// Used to define the different animation types:
use serde::Deserialize;

use serde_json::{self, Value};
use simple_error::{bail, SimpleResult};

#[derive(Deserialize)]
struct Mesh {
    id: String,
    file_type: String,
    dir: String,
}
