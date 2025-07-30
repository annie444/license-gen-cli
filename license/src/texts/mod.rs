pub mod apache;
pub mod bsd;
pub mod bsl;
pub mod cddl;
pub mod epl;
pub mod gnu;
pub mod mit;
pub mod mpl;
pub mod unlicense;

pub use apache::generate_apache_license;
pub use bsd::generate_bsd_license;
pub use bsl::generate_bsl_license;
pub use cddl::generate_cddl_license;
pub use epl::generate_epl_license;
pub use gnu::generate_agpl_license;
pub use gnu::generate_gpl_license;
pub use gnu::generate_lgpl_license;
pub use mit::generate_mit_license;
pub use mpl::generate_mpl_license;
pub use unlicense::generate_unlicense_license;

use std::fmt;

#[derive(Debug)]
pub struct LicenseTexts {
    pub text: String,
    pub comment: String,
    pub alt: Option<String>,
    pub interactive: Option<String>,
}

impl fmt::Display for LicenseTexts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = if self.text.len() > 10 {
            &self.text[0..10]
        } else {
            &self.text
        };
        let comment = if self.comment.len() > 10 {
            &self.comment[0..10]
        } else {
            &self.comment
        };
        let alt = match &self.alt {
            Some(alt) if alt.len() > 10 => &alt[0..10],
            Some(alt) => alt,
            None => "N/A",
        };
        let interactive = match &self.interactive {
            Some(interactive) if interactive.len() > 10 => &interactive[0..10],
            Some(interactive) => interactive,
            None => "N/A",
        };

        write!(
            f,
            "Text: '{text}', Comment: '{comment}', Alternative Text: '{alt}', Interactive Text: '{interactive}'",
        )
    }
}
