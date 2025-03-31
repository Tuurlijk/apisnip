use regex;

fn main() {
    let git_output = std::process::Command::new("git")
        .args(["describe", "--always", "--tags", "--long", "--dirty"])
        .output()
        .ok();
    let git_info = git_output
        .as_ref()
        .and_then(|output| std::str::from_utf8(&output.stdout).ok().map(str::trim));
    let cargo_pkg_version = env!("CARGO_PKG_VERSION");

    // Default git_describe to cargo_pkg_version
    let mut git_describe = String::from(cargo_pkg_version);

    if let Some(git_info) = git_info {
        if git_info.contains(cargo_pkg_version) {
            // Remove the 'g' only if followed by at least 7 hexadecimal characters
            let git_info = regex::Regex::new(r"g([0-9a-f]{7,})")
                .unwrap()
                .replace(git_info, |caps: &regex::Captures| {
                    caps.get(1).unwrap().as_str().to_string()
                });
            git_describe = git_info.to_string();
        } else {
            git_describe = format!("v{}-{}", cargo_pkg_version, git_info);
        }
    }
    println!("cargo:rustc-env=GIT_INFO={}", git_describe);
}
