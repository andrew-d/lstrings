use std::convert::AsRef;


#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct BigramMap {
    // Rather than a HashMap<Bigram, HashMap<u8, u32>>, we can just store
    // our counts as a giant array, indexed by a u16 (which is our Bigram).
    lut: Vec<u32>,
    length: f64,
}

impl BigramMap {
    pub fn new() -> BigramMap {
        // Create an empty vector of size 2 ^ 16.
        let mut lut = Vec::with_capacity(65536);
        lut.resize(65536, 0);

        BigramMap {
            lut:    lut,
            length: 0.0,
        }
    }

    pub fn from_str<S: AsRef<str>>(s: S) -> BigramMap {
        let mut m = BigramMap::new();
        m.add(s);
        m
    }

    pub fn add<S: AsRef<str>>(&mut self, s: S) {
        let s = s.as_ref();

        if s.len() == 0 {
            return
        }

        // Skip all spaces in the string - Wikipedia recommends this as a way
        // to get better accuracy?
        let b = s.as_bytes()
            .into_iter()
            .filter(|&b| *b != b' ')
            .map(|b| *b)
            .collect::<Vec<u8>>();
        for chunk in b.windows(2) {
            self.bump(chunk[0], chunk[1]);
        }

        // The first and final characters get added to a space to use
        self.bump(b' ', b[0]);

        // Don't double-add a single character.
        if b.len() > 1 {
            self.bump(b[b.len() - 1], b' ');
        }

        self.measure();
    }

    // Increment the count for the given index.
    fn bump(&mut self, one: u8, two: u8) {
        let idx = ((one as usize) << 8) | two as usize;
        self.lut[idx] += 1;
    }

    fn measure(&mut self) {
        let mut total: f64 = 0.0;

        // Iterate over all counts and calculate the sum of squares.
        for count in self.lut.iter() {
            total += (count * count) as f64;
        }

        self.length = total.sqrt();
    }

    /// Calculate the similarity between this BigramMap and the other.  Returns
    /// a number between 0.0 and 1.0, where 1.0 indicates an identical ratio of
    /// bigrams, and 0.0 indicates no similarity.
    pub fn similarity(&self, other: &BigramMap) -> f64 {
        let mut total = 0.0;

        // For each possible bigram, we create our sum.
        for bg in 0..65535 {
            total += (self.lut[bg] * other.lut[bg]) as f64;
        }

        total / (self.length * other.length)
    }
}

#[test]
fn test_similarity() {
    let mut bg1 = BigramMap::new();
    bg1.add("foo");
    bg1.add("longer string 123");

    let mut bg2 = BigramMap::new();
    bg2.add("foo");
    bg2.add("longer string 123");

    assert!(bg1.similarity(&bg2).abs_sub(1.0) < 1e-10);
}

#[test]
fn test_dissimilarity() {
    let mut bg1 = BigramMap::new();
    bg1.add("foo");
    bg1.add("bar");

    let mut bg2 = BigramMap::new();
    bg2.add("qqq");
    bg2.add("zzzzz long");

    assert!(bg1.similarity(&bg2) < 1e-10);
}
