use crate::sampler::Sampler;
use crate::film::Film;

// The ThreadData struct has all of the data that is unique to a single thread. So this includes
// its own sampler and whatnot.
pub struct ThreadData<'a, S: Sampler> {
    // The sampler that this thread gets access to:
    sampler: S,

    // The film that this thread uses to 
    film: &'a Film,
}