use const_format::formatcp;

macro_rules! const_str {
    ($($x:ident),*) => {
        $(
            pub const $x: &'static str = env!(stringify!($x));
        )*
    };
}

macro_rules! const_bool {
    ($($x:ident),*) => {
        $(
            pub const $x: bool = const {
                match env!(stringify!($x)).as_bytes() {
                    b"1" => true,
                    _ => false
                }
            };
        )*
    };
}

// Passed from the build script.
const_str!(
    GIT_COMMIT,
    BUILD_DATE,
    BUILD_USER,
    BUILD_PROFILE,
    RUSTC_VERSION
);
const_bool!(IS_VCS_DIRTY, BUILD_IS_DEBUG);

pub const CRATE_NAME: &str = env!("CARGO_PKG_NAME");
pub const CRATE_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const CRATE_SEM_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CRATE_REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

pub const SHORT_VERSION: &str = const {
    formatcp!(
        "{CRATE_SEM_VERSION}-{BUILD_PROFILE} ({GIT_COMMIT}{dirty}) ",
        dirty = if IS_VCS_DIRTY { "+" } else { "" }
    )
};

pub const LONG_VERSION: &str = const {
    formatcp!(
        "{SHORT_VERSION}
 ► built at {BUILD_DATE}
 ► by {BUILD_USER}
 ► with {RUSTC_VERSION}"
    )
};

pub const DEFAULT_FORMATTER: &str = "clang-format";
pub const DEFAULT_CONFIG_FILES: &[&str] = &[".cbundl.toml", "cbundl.toml"];
