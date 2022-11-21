use semver::Version;
use serde::Deserialize;
use std::process::Command;
use std::time::{Instant};
use std::env;

#[derive(Deserialize)]
struct DependencyReportBody {
    body: Vec<Vec<String>>,
}

#[derive(Deserialize)]
struct DependencyReport {
    data: DependencyReportBody,
}

struct AnalysisResult {
    outdated_major: Vec<String>,
    outdated_minor: Vec<String>,
    outdated_patch: Vec<String>,
    sum: usize,
}

fn analyze_dependency_report(report: &Vec<Vec<String>>) -> AnalysisResult {
    let mut outdated_major: Vec<String> = Vec::new();
    let mut outdated_minor: Vec<String> = Vec::new();
    let mut outdated_patch: Vec<String> = Vec::new();
    'analyze: for package in report {
        let name = &package[0];
        let current = &package[1];
        let latest = &package[3];
        if current == latest {
            println!("Package {} is up-to-date. Skipping...", name);
            continue;
        }

        let current = match Version::parse(current) {
            Ok(v) => v,
            Err(e) => {
                println!("Error parsing current version for package {}: {}", name, e);
                continue 'analyze;
            }
        };

        let latest = match Version::parse(latest) {
            Ok(v) => v,
            Err(e) => {
                println!("Error parsing latest version for package {}: {}", name, e);
                continue 'analyze;
            }
        };

        if current.major != latest.major {
            outdated_major.push(name.to_string());
            continue;
        }

        if current.minor != latest.minor {
            outdated_minor.push(name.to_string());
            continue;
        }

        if current.patch != latest.patch {
            outdated_patch.push(name.to_string());
        }
    }

    return AnalysisResult {
        outdated_major,
        outdated_minor,
        outdated_patch,
        sum: report.len(),
    };
}

fn print_table(analysis_result: &AnalysisResult) {
    let outdated_major = analysis_result.outdated_major.len() as f32;
    let outdated_minor = analysis_result.outdated_minor.len() as f32;
    let outdated_patch = analysis_result.outdated_patch.len() as f32;

    let overall_packages = analysis_result.sum as f32;
    let outdated_percentage =
        ((outdated_major + outdated_minor + outdated_patch as f32) / overall_packages) * 100.0;
    println!();
    println!("{:^8} | {:^5} | {:^3}", "", "count", "%");
    println!("{:-^22}", "");
    println!(
        "{:>8} | {:>5} | {:^3.0}",
        "major",
        outdated_major,
        (outdated_major as f32) / overall_packages * 100.0
    );
    println!(
        "{:>8} | {:>5} | {:^3.0}",
        "minor",
        outdated_minor,
        (outdated_minor as f32) / overall_packages * 100.0
    );
    println!(
        "{:>8} | {:>5} | {:^3.0}",
        "patch",
        outdated_patch,
        (outdated_patch as f32) / overall_packages * 100.0
    );
    println!("{:-^22}", "");
    println!(
        "{:>8} | {:>5} | {:^3.0}",
        "overall", overall_packages, outdated_percentage
    );
}

fn run_yarn_outdated(path: &str) -> String {
    let yarn_outdated = Command::new("yarn")
        .arg("outdated")
        .arg("--json")
        .current_dir(path)
        .output()
        .expect("Failed to run yarn");

    let json_output = String::from_utf8(yarn_outdated.stdout).expect("Can't parse stdout!");
    let json_output = json_output.lines().collect::<Vec<&str>>()[1];
    return json_output.to_string();
}

fn parse_working_directory() -> String {
    let args: Vec<String> = env::args().collect();
    return args[1].to_string();
}

fn main() {
    let working_directory = parse_working_directory();

    print!("Running `yarn outdated`... ");
    let start = Instant::now();
    let yarn_output = run_yarn_outdated(&working_directory);
    println!("{:.2?}", start.elapsed());

    let report: DependencyReport =
        serde_json::from_str(&yarn_output).expect("Can't parse JSON output from yarn!");

    let analysis_result = analyze_dependency_report(&report.data.body);
    print_table(&analysis_result);
}
