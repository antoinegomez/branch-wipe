use std::process::Command;
use std::str;
use std::vec::IntoIter;

pub fn list_branches(directory: Option<&String>) -> IntoIter<String> {
  let output = Command::new("git")
    .current_dir(directory.unwrap())
    .arg("branch")
    .arg("--format=%(refname:short)")
    .output()
    .expect("failed to execute process");

  let lines = output.stdout
    .iter()
    .map(|&c| c as char)
    .collect::<String>();

  let debug: Vec<String> = lines
    .split("\n")
    .map(|c| c.trim())
    .filter(|&c| c != "")
    .map(|c| c.to_string())
    .collect();
  debug.into_iter()
}

pub fn delete_branch(directory: Option<&String>, branch: Option<&str>) {
  println!("Directory is {} and branch is {}", directory.unwrap(), branch.unwrap());
  let output = Command::new("git")
    .current_dir(directory.unwrap())
    .arg("branch")
    .arg("-D")
    .arg(branch.unwrap())
    .output()
    .expect("failed to execute process");
  println!("{:?}", output.stdout.iter().map(|&c| c as char).collect::<String>());
  println!("{:?}", output.stderr.iter().map(|&c| c as char).collect::<String>());
}
