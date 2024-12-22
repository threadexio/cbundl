use std::env;
use std::ffi::OsStr;
use std::process::Command;
use std::str::from_utf8;
use std::sync::LazyLock;

fn set_env(key: &str, value: &str) {
    println!("cargo:rustc-env={key}={value}");
}

fn set_env_bool(key: &str, value: bool) {
    set_env(key, if value { "1" } else { "0" });
}

fn rerun_if_changed(path: &str) {
    println!("cargo:rustc-rerun-if-changed={path}");
}

fn run<S, I>(program: &str, args: I) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let p = Command::new(program)
        .args(args)
        .output()
        .expect(&format!("missing tool '{program}'"));

    if p.status.success() {
        Some(from_utf8(&p.stdout).unwrap().trim().to_owned())
    } else {
        None
    }
}

macro_rules! cached_getter {
    ($fn:ident, $cached_type:ty, $typ:ty, $init_fn:block) => {
        pub fn $fn() -> &'static $typ {
            static CACHE: LazyLock<$cached_type> = LazyLock::new(|| $init_fn);
            &CACHE
        }
    };
}

cached_getter!(commit_hash, String, str, {
    run("git", ["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into())
});

cached_getter!(build_date, String, str, {
    let date = chrono::Local::now();
    date.format("%a %d %b %Y %H:%M:%S (UTC%:z)").to_string()
});

cached_getter!(build_user, String, str, {
    let user = run("id", ["-un"]).unwrap_or_else(|| "unknown".into());
    let host = run::<&str, _>("hostname", []).unwrap_or_else(|| "unknown".into());

    format!("{user}@{host}")
});

cached_getter!(build_profile, String, str, { env::var("PROFILE").unwrap() });

cached_getter!(rustc, String, str, { env::var("RUSTC").unwrap() });

cached_getter!(rustc_version, String, str, {
    run(rustc(), ["--version"]).unwrap_or_else(|| "<rustc unknown>".into())
});

pub fn is_vcs_dirty() -> bool {
    static CACHE: LazyLock<bool> = LazyLock::new(|| {
        let git = Command::new("git")
            .args(["diff-index", "--quiet", "HEAD"])
            .output()
            .unwrap();

        git.status.code().unwrap_or(1) == 1
    });

    *CACHE
}

pub fn build_is_debug() -> bool {
    build_profile() == "debug"
}

fn main() {
    rerun_if_changed(".git/HEAD");

    set_env("GIT_COMMIT", commit_hash());
    set_env("BUILD_DATE", build_date());
    set_env("BUILD_USER", build_user());
    set_env("BUILD_PROFILE", build_profile());
    set_env("RUSTC_VERSION", rustc_version());
    set_env_bool("IS_VCS_DIRTY", is_vcs_dirty());
    set_env_bool("BUILD_IS_DEBUG", build_is_debug());
}
