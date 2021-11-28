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

use std::io::{stdout, Write};

use anyhow::Context;
use crossterm::execute;
use crossterm::style::Attribute::Reset;
use crossterm::style::Color::{Green, Red};
use crossterm::style::{
    Attribute, Print, ResetColor, SetAttribute, SetAttributes, SetForegroundColor,
};
use crossterm::tty::IsTty;

use crate::utils::errors::ArchResult;

pub mod theme;

pub type StatusCallback = Box<dyn Fn(&str, bool) -> ArchResult<()>>;

pub fn write_check_ln(text: &str, attributes: &[Attribute]) -> ArchResult<StatusCallback> {
    let is_tty = stdout().is_tty();

    let attributes = attributes.into();

    if is_tty {
        execute!(
            stdout(),
            SetAttributes(attributes),
            Print(text),
            SetAttribute(Reset),
            Print(" ")
        )
        .context("failed to write status text to stdout")?;
    } else {
        write!(stdout(), "{} ", text).context("failed to write status text to stdout")?;
    }

    Ok(Box::new(move |label, success| {
        if is_tty {
            execute!(
                stdout(),
                SetAttributes(attributes),
                SetForegroundColor(if success { Green } else { Red }),
                Print(label),
                ResetColor,
                SetAttribute(Reset),
                Print("\n")
            )
            .context("failed to write status to stdout")
        } else {
            println!("{}", label);

            Ok(())
        }
    }))
}
