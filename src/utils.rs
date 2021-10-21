/*
 * BSD 3-Clause License
 *
 * Copyright (c) 2021, Alex Katlein
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * 1. Redistributions of source code must retain the above copyright notice, this
 *    list of conditions and the following disclaimer.
 *
 * 2. Redistributions in binary form must reproduce the above copyright notice,
 *    this list of conditions and the following disclaimer in the documentation
 *    and/or other materials provided with the distribution.
 *
 * 3. Neither the name of the copyright holder nor the names of its
 *    contributors may be used to endorse or promote products derived from
 *    this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
 * FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
 * CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
 * OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */

use lazy_static::lazy_static;
use regex::Regex;

pub mod constants {
    pub const NO_HISTORY: &'static str = "no-history";
    pub const NO_INIT: &'static str = "no-init";
    pub const IGNORE_CHECKS: &'static str = "ignore-checks";
    pub const VERBOSE: &'static str = "verbose";
}

pub mod reader {
    use std::env::var;
    use std::fs::File;
    use std::io;
    use std::io::BufRead;
    use std::path::Path;
    use std::rc::Rc;
    use std::str::FromStr;

    use lazy_static::lazy_static;

    lazy_static! {
        static ref LINE_BUFFER_CAPACITY: usize =
            if let Ok(Ok(cap)) = var("LINE_BUFFER_CAPACITY").map(|raw| usize::from_str(&raw)) {
                cap
            } else {
                // this is already huge, most source code has lines that are only in the 80-100 characters range
                256
            };
    }

    pub struct BufReader {
        reader: io::BufReader<File>,
        buf: Rc<String>,
    }

    fn new_buf() -> Rc<String> {
        Rc::new(String::with_capacity(*LINE_BUFFER_CAPACITY))
    }

    impl BufReader {
        pub fn open<T: AsRef<Path>>(path: T) -> io::Result<Self> {
            let file = File::open(path)?;
            let reader = io::BufReader::new(file);
            let buf = new_buf();

            Ok(Self { reader, buf })
        }
    }

    impl Iterator for BufReader {
        type Item = io::Result<Rc<String>>;

        fn next(&mut self) -> Option<Self::Item> {
            let buf = match Rc::get_mut(&mut self.buf) {
                Some(buf) => {
                    buf.clear();
                    buf
                }
                None => {
                    self.buf = new_buf();
                    Rc::make_mut(&mut self.buf)
                }
            };

            self.reader
                .read_line(buf)
                .map(|u| {
                    if u == 0 {
                        None
                    } else {
                        Some(Rc::clone(&self.buf))
                    }
                })
                .transpose()
        }
    }

    #[cfg(test)]
    mod tests {
        // todo: test_buf_reader
    }
}

pub struct ToolConfig {
    pub no_history: bool,
    pub no_init: bool,
    pub ignore_checks: bool,
    pub verbose: bool,
}

lazy_static! {
    static ref ID_REGEX: Regex = Regex::new("^[a-zA-Z_$][a-zA-Z0-9_$]*$").unwrap();
    pub static ref NEW_LINE_REGEX: Regex = Regex::new(r#"(\r?\n)(\s+|\r?\n)*"#).unwrap();
}

pub fn is_identifier(value: &str) -> bool {
    ID_REGEX.is_match(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_identifier() {
        assert!(is_identifier("this_is_an_identifier_1$"));
        assert!(!is_identifier("1not_an_identifier"));
    }
}
