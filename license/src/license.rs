use crate::texts;
use clap::ValueEnum;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VersionAmmendment {
    None,
    /// -only
    Only,
    /// -or-later
    OrLater,
}

impl fmt::Display for VersionAmmendment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionAmmendment::None => Ok(()),
            VersionAmmendment::Only => write!(f, "-only"),
            VersionAmmendment::OrLater => write!(f, "-or-later"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BsdAmmendment {
    None,
    /// -Attribution
    Attribution,
    /// -Modification
    Modification,
    /// -No-Military-License
    NoMilitary,
}

impl fmt::Display for BsdAmmendment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BsdAmmendment::None => Ok(()),
            BsdAmmendment::Attribution => write!(f, "-Attribution"),
            BsdAmmendment::Modification => write!(f, "-Modification"),
            BsdAmmendment::NoMilitary => write!(f, "-No-Military-License"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Licenses {
    Mit,
    Agpl3(VersionAmmendment),
    Gpl3(VersionAmmendment),
    Lgpl3(VersionAmmendment),
    Apache2,
    Bsl1,
    Unlicense,
    Cddl1,
    Epl2,
    Mpl2,
    Bsd3Clause(BsdAmmendment),
}

impl fmt::Display for Licenses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Licenses::Mit => write!(f, "MIT"),
            Licenses::Agpl3(a) => write!(f, "AGPL-3.0").and_then(|_| write!(f, "{a}")),
            Licenses::Gpl3(a) => write!(f, "GPL-3.0").and_then(|_| write!(f, "{a}")),
            Licenses::Lgpl3(a) => write!(f, "LGPL-3.0").and_then(|_| write!(f, "{a}")),
            Licenses::Apache2 => write!(f, "Apache-2.0"),
            Licenses::Bsl1 => write!(f, "BSL-1.0"),
            Licenses::Unlicense => write!(f, "Unlicense"),
            Licenses::Cddl1 => write!(f, "CDDL-1.0"),
            Licenses::Epl2 => write!(f, "EPL-2.0"),
            Licenses::Mpl2 => write!(f, "MPL-2.0"),
            Licenses::Bsd3Clause(a) => write!(f, "BSD-3-Clause").and_then(|_| write!(f, "{a}")),
        }
    }
}

impl ValueEnum for Licenses {
    #[tracing::instrument]
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Licenses::Mit,
            Licenses::Agpl3(VersionAmmendment::None),
            Licenses::Agpl3(VersionAmmendment::Only),
            Licenses::Agpl3(VersionAmmendment::OrLater),
            Licenses::Gpl3(VersionAmmendment::None),
            Licenses::Gpl3(VersionAmmendment::Only),
            Licenses::Gpl3(VersionAmmendment::OrLater),
            Licenses::Lgpl3(VersionAmmendment::None),
            Licenses::Lgpl3(VersionAmmendment::Only),
            Licenses::Lgpl3(VersionAmmendment::OrLater),
            Licenses::Apache2,
            Licenses::Bsl1,
            Licenses::Unlicense,
            Licenses::Cddl1,
            Licenses::Epl2,
            Licenses::Mpl2,
            Licenses::Bsd3Clause(BsdAmmendment::None),
            Licenses::Bsd3Clause(BsdAmmendment::Attribution),
            Licenses::Bsd3Clause(BsdAmmendment::Modification),
            Licenses::Bsd3Clause(BsdAmmendment::NoMilitary),
        ]
    }

    #[tracing::instrument]
    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(self.to_string().into())
    }
}

#[tracing::instrument]
pub fn generate_license_text(license: &Licenses) -> texts::LicenseTexts {
    match license {
        Licenses::Mit => texts::generate_mit_license(),
        Licenses::Agpl3(a) => texts::generate_agpl_license(a.clone()),
        Licenses::Gpl3(a) => texts::generate_gpl_license(a.clone()),
        Licenses::Lgpl3(a) => texts::generate_lgpl_license(a.clone()),
        Licenses::Apache2 => texts::generate_apache_license(),
        Licenses::Bsl1 => texts::generate_bsl_license(),
        Licenses::Unlicense => texts::generate_unlicense_license(),
        Licenses::Cddl1 => texts::generate_cddl_license(),
        Licenses::Epl2 => texts::generate_epl_license(),
        Licenses::Mpl2 => texts::generate_mpl_license(),
        Licenses::Bsd3Clause(a) => texts::generate_bsd_license(a.clone()),
    }
}
