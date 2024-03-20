use std::process::{Command, Stdio};

static TESTS: [(&str, &str); 1] = [(
    "-f ./sample_inputs/one.el -n 8",
    "./ok/connected_components-one.el-8.ok",
)];

static MODES: [&str; 2] = ["reader", "hybrid"];

static BUILDS: [&str; 2] = ["dev", "release"];

fn run_test_expected_output(opt: String, input: String, ok_file: String) -> Result<(), Vec<u8>> {
    let input_vec: Vec<String> = input
        .clone()
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect();
    let test_out = Command::new("cargo")
        .args([
            "run",
            "--profile",
            &opt.clone(),
            "--bin",
            "connected_components",
            "--",
        ])
        .args(input_vec)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| {
            panic!(
                "Failed to create connected components with {} in {}",
                &input, &opt
            )
        })
        .stdout
        .expect("Failed to open stdout of connected_components");

    let checker_out = Command::new("diff")
        .args(["-w", "-", &ok_file.clone()])
        .stdin(Stdio::from(test_out))
        .output()
        .expect(&("Failed to finish diff with ".to_owned() + &ok_file));

    if checker_out.status.success() {
        Ok(())
    } else {
        Err(checker_out.stderr)
    }
}

#[test]
pub fn connected_component_tests() {
    for (input, ok_file) in TESTS {
        for mode in MODES {
            for build in BUILDS {
                match run_test_expected_output(
                    build.to_owned(),
                    input.to_owned() + " --mode " + mode,
                    ok_file.to_owned(),
                ) {
                    Ok(()) => {}
                    Err(vec) => {
                        eprintln!("{}", std::str::from_utf8(&vec).unwrap());
                        panic!(
                            "Failed test with input: {}, and ok_file: {} in build {}",
                            input, ok_file, build
                        );
                    }
                }
            }
        }
    }
}
