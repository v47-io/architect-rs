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

use std::fmt;
use std::fmt::Write;

use crossterm::style::Stylize;
use dialoguer::theme::Theme;

pub struct ArchTheme;

pub(crate) const FORMAT_SEPARATOR: &str = "#|#";

pub(crate) const INSTANCE: ArchTheme = ArchTheme {};

pub(crate) trait WithFormat {
    fn with_format(self, format: &str) -> String;
}

impl WithFormat for String {
    fn with_format(self, format: &str) -> String {
        format!("{}{}{}", self, FORMAT_SEPARATOR, format)
    }
}

trait PromptPunctuation {
    fn has_punctuation(&self) -> bool;
}

impl PromptPunctuation for str {
    fn has_punctuation(&self) -> bool {
        static CHARS: [char; 2] = [':', '?'];

        if let Some(last) = self.chars().rev().next() {
            CHARS.contains(&last)
        } else {
            false
        }
    }
}

impl Theme for ArchTheme {
    #[inline]
    fn format_prompt(&self, f: &mut dyn Write, prompt: &str) -> fmt::Result {
        if prompt.has_punctuation() {
            write!(f, "{} {}", "?".yellow(), prompt.bold())
        } else {
            write!(f, "{} {}:", "?".yellow(), prompt.bold())
        }
    }

    #[inline]
    fn format_error(&self, f: &mut dyn Write, err: &str) -> fmt::Result {
        write!(f, "{} Error: {}", "???".red(), err.red())
    }

    fn format_confirm_prompt(
        &self,
        f: &mut dyn Write,
        prompt: &str,
        default: Option<bool>,
    ) -> fmt::Result {
        if !prompt.is_empty() {
            write!(f, "{} {} ", "?".yellow(), prompt.bold())?;
        }

        let has_punctuation = prompt.has_punctuation();

        match default {
            None => write!(f, "[y/n]{} ", if has_punctuation { "" } else { ":" }),
            Some(true) => write!(
                f,
                "[{}/n]{} ",
                "Y".bold(),
                if has_punctuation { "" } else { ":" }
            ),
            Some(false) => write!(
                f,
                "[y/{}]{} ",
                "N".bold(),
                if has_punctuation { "" } else { ":" }
            ),
        }
    }

    fn format_confirm_prompt_selection(
        &self,
        f: &mut dyn Write,
        prompt: &str,
        selection: Option<bool>,
    ) -> fmt::Result {
        let selection = selection.map(|it| if it { "Yes".green() } else { "No".red() }.bold());

        match selection {
            Some(selection) if prompt.is_empty() => write!(f, "{}", selection),
            Some(selection) => write!(
                f,
                "{} {}{} {}",
                "???".green(),
                prompt,
                if prompt.has_punctuation() { "" } else { ":" },
                selection
            ),
            None if prompt.is_empty() => Ok(()),
            None => write!(f, "{}", prompt),
        }
    }

    fn format_input_prompt(
        &self,
        f: &mut dyn Write,
        prompt: &str,
        default: Option<&str>,
    ) -> fmt::Result {
        let (prompt, format) = if let Some(sep_i) = prompt.find(FORMAT_SEPARATOR) {
            let format = prompt[sep_i + FORMAT_SEPARATOR.len()..].trim();

            (&prompt[..sep_i], Some(format))
        } else {
            (prompt, None)
        };

        let has_punctuation = prompt.has_punctuation();
        let prompt = prompt.bold();

        write!(f, "{} {}", "?".yellow(), prompt)?;

        if let Some(format) = format {
            write!(f, " ({})", format)?;
        }

        if let Some(default) = default {
            write!(f, " [{}]", default.dim())?;
        }

        write!(f, "{} ", if has_punctuation { "" } else { ":" })
    }

    fn format_input_prompt_selection(
        &self,
        f: &mut dyn Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        let prompt = if let Some(sep_i) = prompt.find(FORMAT_SEPARATOR) {
            &prompt[..sep_i]
        } else {
            prompt
        };

        write!(
            f,
            "{} {}{} {}",
            "???".green(),
            prompt,
            if prompt.has_punctuation() { "" } else { ":" },
            sel.bold()
        )
    }

    fn format_password_prompt(&self, f: &mut dyn Write, prompt: &str) -> fmt::Result {
        self.format_input_prompt(f, prompt, None)
    }

    fn format_password_prompt_selection(&self, f: &mut dyn Write, prompt: &str) -> fmt::Result {
        self.format_input_prompt_selection(f, prompt, "***")
    }

    fn format_select_prompt(&self, f: &mut dyn Write, prompt: &str) -> fmt::Result {
        self.format_prompt(f, prompt)
    }

    fn format_select_prompt_selection(
        &self,
        f: &mut dyn Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        self.format_input_prompt_selection(f, prompt, sel)
    }

    fn format_multi_select_prompt(&self, f: &mut dyn Write, prompt: &str) -> fmt::Result {
        self.format_prompt(f, prompt)
    }

    fn format_sort_prompt(&self, f: &mut dyn Write, prompt: &str) -> fmt::Result {
        self.format_prompt(f, prompt)
    }

    fn format_multi_select_prompt_selection(
        &self,
        f: &mut dyn Write,
        prompt: &str,
        selections: &[&str],
    ) -> fmt::Result {
        let selection = if !selections.is_empty() {
            selections.join(", ")
        } else {
            "[nothing selected]".dim().to_string()
        };

        self.format_input_prompt_selection(f, prompt, &selection)
    }

    fn format_sort_prompt_selection(
        &self,
        f: &mut dyn Write,
        prompt: &str,
        selections: &[&str],
    ) -> fmt::Result {
        self.format_multi_select_prompt_selection(f, prompt, selections)
    }

    fn format_select_prompt_item(
        &self,
        f: &mut dyn Write,
        text: &str,
        active: bool,
    ) -> fmt::Result {
        write!(f, "{} {}", if active { "> " } else { "  " }, text)
    }

    fn format_multi_select_prompt_item(
        &self,
        f: &mut dyn Write,
        text: &str,
        checked: bool,
        active: bool,
    ) -> fmt::Result {
        write!(
            f,
            "{} {}",
            match (checked, active) {
                (true, true) => "> [x]".bold(),
                (true, false) => "  [x]".bold(),
                (false, true) => "> [ ]".stylize(),
                (false, false) => "  [ ]".dim(),
            },
            match (checked, active) {
                (true, true) => text.bold(),
                (true, false) => text.bold(),
                (false, true) => text.stylize(),
                (false, false) => text.dim(),
            }
        )
    }

    fn format_sort_prompt_item(
        &self,
        f: &mut dyn Write,
        text: &str,
        picked: bool,
        active: bool,
    ) -> fmt::Result {
        write!(
            f,
            "{} {}",
            match (picked, active) {
                (true, true) => "> [x]".bold(),
                (false, true) => "> [ ]".stylize(),
                (_, false) => "  [ ]".dim(),
            },
            match (picked, active) {
                (true, true) => text.bold(),
                (false, true) => text.stylize(),
                (_, false) => text.dim(),
            },
        )
    }
}
