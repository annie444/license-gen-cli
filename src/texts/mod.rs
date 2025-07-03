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

pub struct LicenseTexts {
    pub text: String,
    pub comment: String,
    pub alt: Option<String>,
    pub interactive: Option<String>,
}
