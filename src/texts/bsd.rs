use super::LicenseTexts;
use crate::io::{prompt, prompt_optional};
use crate::license::BsdAmmendment;
use color_print::ceprintln;
use handlebars::Handlebars;
use serde::Serialize;
use std::process;

#[tracing::instrument]
pub fn generate_bsd_license(sublicense: BsdAmmendment) -> LicenseTexts {
    let year: u16 = prompt("Enter the copyright year");
    let fullname: String = prompt("Enter the full name of the copyright holder");
    match sublicense {
        BsdAmmendment::None => generate_base_license(year, fullname),
        BsdAmmendment::Attribution => generate_attribution_license(year, fullname),
        BsdAmmendment::Modification => generate_modification_license(year, fullname),
        BsdAmmendment::NoMilitary => generate_no_military_license(year, fullname),
    }
}

#[tracing::instrument]
pub fn generate_base_license(year: u16, fullname: String) -> LicenseTexts {
    let license = BsdLicenseTemplate {
        year,
        fullname,
        organization: None,
        website: None,
        license: NONE,
    };
    let mut handlebars = Handlebars::new();
    match handlebars.register_template_string("bsd_license", TEXT) {
        Ok(_) => {}
        Err(e) => {
            ceprintln!("<bold><red>Error registering template</></>: {}", e);
            process::exit(1);
        }
    }
    match handlebars.register_partial("fourth_partial", NONE.fourth) {
        Ok(_) => {}
        Err(e) => {
            ceprintln!("<bold><red>Error registering partial</></>: {}", e);
            process::exit(1);
        }
    }
    let text = match handlebars.render("bsd_license", &license) {
        Ok(t) => t,
        Err(e) => {
            ceprintln!("<bold><red>Error rendering template</></>: {}", e);
            process::exit(1);
        }
    };

    LicenseTexts {
        text,
        comment: "SPDX-License-Identifier: BSD-3-Clause".to_string(),
        alt: None,
        interactive: None,
    }
}

#[tracing::instrument]
pub fn generate_attribution_license(year: u16, fullname: String) -> LicenseTexts {
    let organization: Option<String> =
        prompt_optional("Enter the name of the organization (optional): ");
    let website: Option<String> =
        prompt_optional("Enter the website of the organization (optional): ");

    let license = BsdLicenseTemplate {
        year,
        fullname,
        organization,
        website,
        license: ATTRIBUTION,
    };
    let mut handlebars = Handlebars::new();
    match handlebars.register_template_string("bsd_license", TEXT) {
        Ok(_) => {}
        Err(e) => {
            ceprintln!("<bold><red>Error registering template</></>: {}", e);
            process::exit(1);
        }
    }
    match handlebars.register_partial("fourth_partial", ATTRIBUTION.fourth) {
        Ok(_) => {}
        Err(e) => {
            ceprintln!("<bold><red>Error registering partial</></>: {}", e);
            process::exit(1);
        }
    }
    let text = match handlebars.render("bsd_license", &license) {
        Ok(t) => t,
        Err(e) => {
            ceprintln!("<bold><red>Error rendering template</></>: {}", e);
            process::exit(1);
        }
    };

    LicenseTexts {
        text,
        comment: "SPDX-License-Identifier: BSD-3-Clause-Attribution".to_string(),
        alt: None,
        interactive: None,
    }
}

#[tracing::instrument]
pub fn generate_modification_license(year: u16, fullname: String) -> LicenseTexts {
    let license = BsdLicenseTemplate {
        year,
        fullname,
        organization: None,
        website: None,
        license: MODIFICATION,
    };
    let mut handlebars = Handlebars::new();
    match handlebars.register_template_string("bsd_license", TEXT) {
        Ok(_) => {}
        Err(e) => {
            ceprintln!("<bold><red>Error registering template</></>: {}", e);
            process::exit(1);
        }
    };
    match handlebars.register_partial("fourth_partial", MODIFICATION.fourth) {
        Ok(_) => {}
        Err(e) => {
            ceprintln!("<bold><red>Error registering partial</></>: {}", e);
            process::exit(1);
        }
    };
    let text = match handlebars.render("bsd_license", &license) {
        Ok(t) => t,
        Err(e) => {
            ceprintln!("<bold><red>Error rendering template</></>: {}", e);
            process::exit(1);
        }
    };

    LicenseTexts {
        text,
        comment: "SPDX-License-Identifier: BSD-3-Clause-Modification".to_string(),
        alt: None,
        interactive: None,
    }
}

#[tracing::instrument]
pub fn generate_no_military_license(year: u16, fullname: String) -> LicenseTexts {
    let license = BsdLicenseTemplate {
        year,
        fullname,
        organization: None,
        website: None,
        license: NO_MILITARY,
    };
    let mut handlebars = Handlebars::new();
    match handlebars.register_template_string("bsd_license", TEXT) {
        Ok(_) => {}
        Err(e) => {
            ceprintln!("<bold><red>Error registering template</></>: {}", e);
            process::exit(1);
        }
    };
    match handlebars.register_partial("fourth_partial", NO_MILITARY.fourth) {
        Ok(_) => {}
        Err(e) => {
            ceprintln!("<bold><red>Error registering partial</></>: {}", e);
            process::exit(1);
        }
    };
    let text = match handlebars.render("bsd_license", &license) {
        Ok(t) => t,
        Err(e) => {
            ceprintln!("<bold><red>Error rendering template</></>: {}", e);
            process::exit(1);
        }
    };

    LicenseTexts {
        text,
        comment: "SPDX-License-Identifier: BSD-3-Clause-No-Military".to_string(),
        alt: None,
        interactive: None,
    }
}

#[derive(Serialize)]
pub struct BsdLicenseTemplate {
    pub year: u16,
    pub fullname: String,
    pub organization: Option<String>,
    pub website: Option<String>,
    pub license: BsdLicenseText,
}

#[derive(Serialize)]
pub struct BsdLicenseText {
    pub fourth: &'static str,
    pub postamble: &'static str,
}

pub const NONE: BsdLicenseText = BsdLicenseText {
    fourth: "",
    postamble: "",
};

pub const NO_MILITARY: BsdLicenseText = BsdLicenseText {
    fourth: "",
    postamble: r#"
YOU ACKNOWLEDGE THAT THIS SOFTWARE IS NOT DESIGNED, LICENSED OR INTENDED 
FOR USE IN THE DESIGN, CONSTRUCTION, OPERATION OR MAINTENANCE OF ANY MILITARY 
FACILITY.
"#,
};

pub const MODIFICATION: BsdLicenseText = BsdLicenseText {
    fourth: r#"4. If any files are modified, you must cause the modified files to 
carry prominent notices stating that you changed the files and the 
date of any change."#,
    postamble: "",
};

pub const ATTRIBUTION: BsdLicenseText = BsdLicenseText {
    fourth: r#"4. Redistributions of any form whatsoever must retain the following 
acknowledgment: 'This product includes software developed by the 
"{{#if organization}}{{organization}}{{else}}{{fullname}}{{/if}}"{{#if website}} ({{website}}){{/if}}.'"#,
    postamble: "",
};

pub const TEXT: &str = r#"Copyright (c) {{year}} {{fullname}}.

Redistribution and use in source and binary forms, with or without 
modification, are permitted provided that the following conditions are 
met:

1. Redistributions of source code must retain the above copyright 
   notice, this list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright 
   notice, this list of conditions and the following disclaimer in the 
   documentation and/or other materials provided with the distribution.
3. Neither the name of the copyright holder nor the names of its 
   contributors may be used to endorse or promote products derived from 
   this software without specific prior written permission.
{{> fourth_partial}}

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS 
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT 
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR 
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE 
COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, 
INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT 
NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, 
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY 
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT 
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF 
THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
{{license.postamble}}
"#;
