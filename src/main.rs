use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use std::fs;
use std::process::Command;
use std::path::PathBuf;
use std::io::{self, Write};
use dirs::home_dir;
use clap::{Parser};

#[derive(Parser, Debug)]
#[command(author, version, about = "AI Programming Agent (AIPA)", long_about = None)]
struct Args {
    #[arg(short, long, help = "Programming language (e.g., rust, python, cpp, java)")]
    language: String,
    #[arg(short, long, help = "Task goal (e.g., 'print hello')")]
    goal: String,
    #[arg(short, long, help = "Enable debug output", default_value_t = false)]
    debug: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    language: String,
    goal: String,
}

struct AIPA {
    project_dir: PathBuf,
    debug: bool,
}

impl AIPA {
    fn new(debug: bool) -> Result<Self> {
        let home = home_dir().context("Could not find home directory")?;
        let project_dir = home.join("aipa_projects");
        fs::create_dir_all(&project_dir)?;
        if debug {
            println!("Project dir: {:?}", project_dir);
        }
        Ok(AIPA { project_dir, debug })
    }

    fn process_task(&self, task: Task) -> Result<String> {
        const MAX_ATTEMPTS: usize = 3;
        let mut filename = self.get_filename(&task);
        let mut code = if !self.project_dir.join(&filename).exists() {
            let initial_code = self.generate_code(&task)?;
            self.save_code(&task, &initial_code)?;
            initial_code
        } else {
            fs::read_to_string(self.project_dir.join(&filename))?
        };

        let mut result_msg = String::new();
        for attempt in 1..=MAX_ATTEMPTS {
            let result = self.execute_code(&task, &filename)?;
            if result.success {
                result_msg = format!("Success! Output: {}", result.output);
                break;
            }

            let error_msg = result.error.unwrap_or("Unknown error".to_string());
            println!("Attempt {}/{} failed: {}", attempt, MAX_ATTEMPTS, error_msg);

            if attempt == MAX_ATTEMPTS {
                result_msg = format!("Error after {} attempts: {}", MAX_ATTEMPTS, error_msg);
                break;
            }

            code = self.prompt_for_fix(&task, &code, &error_msg)?;
            filename = self.save_code(&task, &code)?;
        }

        self.cleanup(&task)?;
        Ok(result_msg)
    }

    fn generate_code(&self, task: &Task) -> Result<String> {
        match task.language.as_str() {
            "rust" => Ok(format!(
                "fn main() {{ println!(\"AIPA: {} completed\"); }}",
                task.goal
            )),
            "python" => Ok(format!("print('AIPA: {} completed')", task.goal)),
            "cpp" => Ok(format!(
                "#include <iostream>\nint main() {{\n    std::cout << \"AIPA: {} completed\" << std::endl;\n    return 0;\n}}",
                task.goal
            )),
            "java" => Ok(format!(
                "public class project_print_hello {{\n    public static void main(String[] args) {{\n        System.out.println(\"AIPA: {} completed\");\n    }}\n}}",
                task.goal
            )),
            _ => Ok(format!("# Unsupported language: {}", task.language)),
        }
    }

    fn prompt_for_fix(&self, task: &Task, old_code: &String, error: &String) -> Result<String> {
        println!("Task: {} in {}", task.goal, task.language);
        println!("Original code:\n{}", old_code);
        println!("Error: {}", error);
        println!("Enter fixed code below (press Enter twice on a blank line to submit):");

        let mut fixed_code = String::new();
        let stdin = io::stdin();
        loop {
            print!("> ");
            io::stdout().flush()?;
            let mut line = String::new();
            stdin.read_line(&mut line)?;
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() && !fixed_code.trim().is_empty() {
                break;
            }
            if !trimmed_line.is_empty() && trimmed_line != ">" && !trimmed_line.starts_with("Enter fixed code") {
                let clean_line = trimmed_line.trim_start_matches("> ").trim_start_matches(">");
                fixed_code.push_str(clean_line);
                fixed_code.push('\n');
            }
        }
        Ok(fixed_code.trim().to_string())
    }

    fn get_filename(&self, task: &Task) -> String {
        let ext = match task.language.as_str() {
            "rust" => "rs",
            "python" => "py",
            "cpp" => "cpp",
            "java" => "java",
            _ => "txt",
        };
        format!("project_{}.{}", task.goal.replace(" ", "_"), ext)
    }

    fn save_code(&self, task: &Task, code: &String) -> Result<String> {
        let filename = self.get_filename(task);
        let filepath = self.project_dir.join(&filename);
        fs::write(&filepath, code)?;
        if self.debug {
            println!("Saved file: {:?}", filepath);
            println!("Saved code:\n{}", code);
        }
        Ok(filename)
    }

    fn cleanup(&self, task: &Task) -> Result<()> {
        let filename = self.get_filename(task);
        let source_path = self.project_dir.join(&filename);
        let binary_path = source_path.with_extension("");
        let java_class_prefix = "project_print_hello";

        if source_path.exists() {
            fs::remove_file(&source_path)?;
            if self.debug {
                println!("Removed source file: {:?}", source_path);
            }
        }
        if binary_path.exists() {
            fs::remove_file(&binary_path)?;
            if self.debug {
                println!("Removed binary: {:?}", binary_path);
            }
        }
        if task.language == "java" {
            for entry in fs::read_dir(&self.project_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() && path.file_stem().map(|s| s.to_string_lossy().starts_with(java_class_prefix)).unwrap_or(false) && path.extension().map(|e| e == "class").unwrap_or(false) {
                    fs::remove_file(&path)?;
                    if self.debug {
                        println!("Removed class file: {:?}", path);
                    }
                }
            }
        }
        Ok(())
    }

    fn execute_code(&self, task: &Task, filename: &String) -> Result<ExecutionResult> {
        let filepath = self.project_dir.join(filename);
        match task.language.as_str() {
            "rust" => {
                let output_path = filepath.with_extension("");
                if output_path.exists() {
                    fs::remove_file(&output_path)?;
                    if self.debug {
                        println!("Removed old binary: {:?}", output_path);
                    }
                }
                if self.debug {
                    println!("Source file: {:?}", filepath);
                    println!("Intended output: {:?}", output_path);
                    println!("Current dir: {:?}", std::env::current_dir()?);
                    println!("Compiling: rustc {} -o {:?}", filepath.display(), output_path.display());
                }
                let compile = Command::new("rustc")
                    .arg(&filepath)
                    .arg("-o")
                    .arg(&output_path)
                    .current_dir(&self.project_dir)
                    .output()?;
                if self.debug {
                    println!("Compile stdout: {}", String::from_utf8_lossy(&compile.stdout));
                    println!("Compile stderr: {}", String::from_utf8_lossy(&compile.stderr));
                    println!("Compile exit code: {:?}", compile.status);
                }
                if !compile.status.success() {
                    return Ok(ExecutionResult {
                        success: false,
                        output: String::new(),
                        error: Some(String::from_utf8_lossy(&compile.stderr).to_string()),
                    });
                }
                if self.debug {
                    println!("Checking for binary at: {:?}", output_path);
                }
                if !output_path.exists() {
                    if self.debug {
                        println!("Binary not found at intended location: {:?}", output_path);
                        let current_dir_binary = std::env::current_dir()?.join("project_print_hello");
                        println!("Checking current dir: {:?}", current_dir_binary);
                        if current_dir_binary.exists() {
                            println!("Binary found in current dir instead: {:?}", current_dir_binary);
                        }
                    }
                    return Ok(ExecutionResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Binary not created at {:?}", output_path)),
                    });
                }
                if self.debug {
                    println!("Binary exists: {:?}", output_path);
                    println!("File metadata: {:?}", fs::metadata(&output_path));
                    let absolute_path = fs::canonicalize(&output_path)?;
                    println!("Absolute path: {:?}", absolute_path);
                    println!("Running binary: {:?}", absolute_path);
                }
                let absolute_path = fs::canonicalize(&output_path)?;
                let run = Command::new(&absolute_path)
                    .output()
                    .map_err(|e| anyhow::anyhow!("Failed to execute binary: {}", e))?;
                Ok(ExecutionResult {
                    success: run.status.success(),
                    output: String::from_utf8_lossy(&run.stdout).to_string(),
                    error: if run.status.success() {
                        None
                    } else {
                        Some(String::from_utf8_lossy(&run.stderr).to_string())
                    },
                })
            }
            "python" => {
                let run = Command::new("python")
                    .arg(&filepath)
                    .output()?;
                Ok(ExecutionResult {
                    success: run.status.success(),
                    output: String::from_utf8_lossy(&run.stdout).to_string(),
                    error: if run.status.success() {
                        None
                    } else {
                        Some(String::from_utf8_lossy(&run.stderr).to_string())
                    },
                })
            }
            "cpp" => {
                let output_path = filepath.with_extension("");
                if output_path.exists() {
                    fs::remove_file(&output_path)?;
                    if self.debug {
                        println!("Removed old binary: {:?}", output_path);
                    }
                }
                if self.debug {
                    println!("Source file: {:?}", filepath);
                    println!("Intended output: {:?}", output_path);
                    println!("Current dir: {:?}", std::env::current_dir()?);
                    println!("Compiling: g++ {} -o {:?}", filepath.display(), output_path.display());
                }
                let compile = Command::new("g++")
                    .arg(&filepath)
                    .arg("-o")
                    .arg(&output_path)
                    .current_dir(&self.project_dir)
                    .output()?;
                if self.debug {
                    println!("Compile stdout: {}", String::from_utf8_lossy(&compile.stdout));
                    println!("Compile stderr: {}", String::from_utf8_lossy(&compile.stderr));
                    println!("Compile exit code: {:?}", compile.status);
                }
                if !compile.status.success() {
                    return Ok(ExecutionResult {
                        success: false,
                        output: String::new(),
                        error: Some(String::from_utf8_lossy(&compile.stderr).to_string()),
                    });
                }
                if self.debug {
                    println!("Checking for binary at: {:?}", output_path);
                }
                if !output_path.exists() {
                    if self.debug {
                        println!("Binary not found at intended location: {:?}", output_path);
                        let current_dir_binary = std::env::current_dir()?.join("project_print_hello");
                        println!("Checking current dir: {:?}", current_dir_binary);
                        if current_dir_binary.exists() {
                            println!("Binary found in current dir instead: {:?}", current_dir_binary);
                        }
                    }
                    return Ok(ExecutionResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Binary not created at {:?}", output_path)),
                    });
                }
                if self.debug {
                    println!("Binary exists: {:?}", output_path);
                    println!("File metadata: {:?}", fs::metadata(&output_path));
                    let absolute_path = fs::canonicalize(&output_path)?;
                    println!("Absolute path: {:?}", absolute_path);
                    println!("Running binary: {:?}", absolute_path);
                }
                let absolute_path = fs::canonicalize(&output_path)?;
                let run = Command::new(&absolute_path)
                    .output()
                    .map_err(|e| anyhow::anyhow!("Failed to execute binary: {}", e))?;
                Ok(ExecutionResult {
                    success: run.status.success(),
                    output: String::from_utf8_lossy(&run.stdout).to_string(),
                    error: if run.status.success() {
                        None
                    } else {
                        Some(String::from_utf8_lossy(&run.stderr).to_string())
                    },
                })
            }
            "java" => {
                let class_name = "project_print_hello";
                let class_file = self.project_dir.join(format!("{}.class", class_name));
                if class_file.exists() {
                    fs::remove_file(&class_file)?;
                    if self.debug {
                        println!("Removed old class file: {:?}", class_file);
                    }
                }
                if self.debug {
                    println!("Source file: {:?}", filepath);
                    println!("Class name: {}", class_name);
                    println!("Current dir: {:?}", std::env::current_dir()?);
                    println!("Compiling: javac {}", filepath.display());
                }
                let compile = Command::new("javac")
                    .arg(&filepath)
                    .current_dir(&self.project_dir)
                    .output()?;
                if self.debug {
                    println!("Compile stdout: {}", String::from_utf8_lossy(&compile.stdout));
                    println!("Compile stderr: {}", String::from_utf8_lossy(&compile.stderr));
                    println!("Compile exit code: {:?}", compile.status);
                }
                if !compile.status.success() {
                    return Ok(ExecutionResult {
                        success: false,
                        output: String::new(),
                        error: Some(String::from_utf8_lossy(&compile.stderr).to_string()),
                    });
                }
                if self.debug {
                    println!("Checking for class file at: {:?}", class_file);
                }
                if !class_file.exists() {
                    if self.debug {
                        println!("Class file not found at: {:?}", class_file);
                    }
                    return Ok(ExecutionResult {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Class file not created at {:?}", class_file)),
                    });
                }
                if self.debug {
                    println!("Class file exists: {:?}", class_file);
                    println!("Running: java {}", class_name);
                }
                let run = Command::new("java")
                    .arg(&class_name)
                    .current_dir(&self.project_dir)
                    .output()?;
                Ok(ExecutionResult {
                    success: run.status.success(),
                    output: String::from_utf8_lossy(&run.stdout).to_string(),
                    error: if run.status.success() {
                        None
                    } else {
                        Some(String::from_utf8_lossy(&run.stderr).to_string())
                    },
                })
            }
            _ => Ok(ExecutionResult {
                success: false,
                output: String::new(),
                error: Some("Unsupported language".to_string()),
            }),
        }
    }
}

#[derive(Debug)]
struct ExecutionResult {
    success: bool,
    output: String,
    error: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let aipa = AIPA::new(args.debug)?;
    
    let task = Task {
        language: args.language,
        goal: args.goal,
    };
    
    let result = aipa.process_task(task)?;
    println!("{}", result);
    
    Ok(())
}
