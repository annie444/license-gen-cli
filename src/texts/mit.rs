use super::LicenseTexts;
use crate::io::prompt;
use color_print::ceprintln;
use handlebars::Handlebars;
use serde::Serialize;
use std::process;

pub fn generate_mit_license() -> LicenseTexts {
    let year: u16 = prompt("Enter the copyright year");
    let fullname: String = prompt("Enter the full name of the copyright holder");

    let license = MitLicenseTemplate { year, fullname };

    let mut handlebars = Handlebars::new();
    match handlebars.register_template_string("mit_license", MIT) {
        Ok(_) => {}
        Err(e) => {
            ceprintln!("Error registering template: {}", e);
            process::exit(1);
        }
    }

    let text = match handlebars.render("mit_license", &license) {
        Ok(rendered) => rendered,
        Err(e) => {
            ceprintln!("Error rendering template: {}", e);
            process::exit(1);
        }
    };

    LicenseTexts {
        text,
        comment: "SPDX-License-Identifier: MIT".to_string(),
        alt: None,
        interactive: None,
    }
}

#[derive(Serialize)]
pub struct MitLicenseTemplate {
    pub year: u16,
    pub fullname: String,
}

pub const MIT: &'static str = r#"MIT License

Copyright (c) {{year}} {{fullname}}

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"#;
