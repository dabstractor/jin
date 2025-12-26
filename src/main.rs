use clap::error::ErrorKind;
use clap::Parser;
use jin_glm::cli::{Cli, Commands, ModeCommand, ScopeCommand};
use jin_glm::commands;
use std::process::ExitCode;

fn main() -> ExitCode {
    match Cli::try_parse() {
        Ok(cli) => {
            // TODO: Dispatch to command handlers (P4.M2-M4)
            // For now, show placeholder messages
            match cli.command {
                // Core Commands
                Commands::Init(cmd) => match commands::execute(&cmd) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::FAILURE
                    }
                },
                Commands::Add(cmd) => match commands::add_execute(&cmd) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::FAILURE
                    }
                },
                Commands::Commit(cmd) => match commands::commit_execute(&cmd) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::FAILURE
                    }
                },
                Commands::Reset(cmd) => match commands::reset_execute(&cmd) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::FAILURE
                    }
                },
                Commands::Status(cmd) => match commands::status_execute(&cmd) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::FAILURE
                    }
                },

                // Mode Management
                Commands::Mode(ModeCommand::Create { name }) => {
                    match commands::mode_execute(&ModeCommand::Create { name }) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Commands::Mode(ModeCommand::Use { name }) => {
                    match commands::mode_execute(&ModeCommand::Use { name }) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Commands::Mode(ModeCommand::Unset) => {
                    match commands::mode_execute(&ModeCommand::Unset) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Commands::Mode(ModeCommand::Delete { name }) => {
                    match commands::mode_execute(&ModeCommand::Delete { name }) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Commands::Mode(ModeCommand::Show) => {
                    match commands::mode_execute(&ModeCommand::Show) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Commands::Modes => match commands::mode_list_execute() {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::FAILURE
                    }
                },

                // Scope Management
                Commands::Scope(ScopeCommand::Create { name, mode }) => {
                    match commands::scope_execute(&ScopeCommand::Create { name, mode }) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Commands::Scope(ScopeCommand::Use { name }) => {
                    match commands::scope_execute(&ScopeCommand::Use { name }) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Commands::Scope(ScopeCommand::Unset) => {
                    match commands::scope_execute(&ScopeCommand::Unset) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Commands::Scope(ScopeCommand::Delete { name }) => {
                    match commands::scope_execute(&ScopeCommand::Delete { name }) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Commands::Scope(ScopeCommand::Show) => {
                    match commands::scope_execute(&ScopeCommand::Show) {
                        Ok(()) => ExitCode::SUCCESS,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            ExitCode::FAILURE
                        }
                    }
                }
                Commands::Scopes => match commands::scope_list_execute() {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::FAILURE
                    }
                },

                // Workspace Operations
                Commands::Apply(cmd) => match commands::apply_execute(&cmd) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::FAILURE
                    }
                },

                // Inspection Commands
                Commands::Diff(cmd) => match commands::diff_execute(&cmd) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::FAILURE
                    }
                },
                Commands::Log(cmd) => match commands::log_execute(&cmd) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::FAILURE
                    }
                },
                Commands::Context => match commands::context_execute() {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::FAILURE
                    }
                },
                Commands::Layers => match commands::layers_execute() {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::FAILURE
                    }
                }
                Commands::List => {
                    println!("jin list - command handler to be implemented");
                    ExitCode::SUCCESS
                }

                // Import/Export
                Commands::Import(cmd) => match commands::import_execute(&cmd) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::FAILURE
                    }
                },
                Commands::Export(cmd) => match commands::export_execute(&cmd) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::FAILURE
                    }
                },

                // Maintenance
                Commands::Repair(cmd) => match commands::repair_execute(&cmd) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        ExitCode::FAILURE
                    }
                },

                // Remote Operations
                Commands::Link(_) => {
                    println!("jin link - command handler to be implemented");
                    ExitCode::SUCCESS
                }
                Commands::Fetch => {
                    println!("jin fetch - command handler to be implemented");
                    ExitCode::SUCCESS
                }
                Commands::Pull => {
                    println!("jin pull - command handler to be implemented");
                    ExitCode::SUCCESS
                }
                Commands::Push => {
                    println!("jin push - command handler to be implemented");
                    ExitCode::SUCCESS
                }
                Commands::Sync => {
                    println!("jin sync - command handler to be implemented");
                    ExitCode::SUCCESS
                }

                // File Operations
                Commands::Rm(_) => {
                    println!("jin rm - command handler to be implemented");
                    ExitCode::SUCCESS
                }
                Commands::Mv(_) => {
                    println!("jin mv - command handler to be implemented");
                    ExitCode::SUCCESS
                }
            }
        }
        Err(e) => {
            // clap automatically displays help/error message - print it
            print!("{e}");
            // Help and version requests should exit successfully
            if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}
