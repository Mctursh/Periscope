//! Pretty-print helpers for CLI output

use crate::idl::{Idl, IdlAccount, IdlAccountItem, IdlInstruction, IdlType, IdlTypeComplex};
use colored::Colorize;

/// Print a main header (program name, command title)
pub fn print_header(title: &str) {
    println!();
    println!("{}", title.bold().cyan());
    println!("{}", "─".repeat(50).dimmed());
}

/// Print a sub-header (section within output)
pub fn print_subheader(title: &str) {
    println!();
    println!("{}", title.bold().white());
}

/// Print a key-value pair
pub fn print_field(key: &str, value: &str) {
    println!("  {}: {}", key.dimmed(), value);
}

/// Print a numbered list item
pub fn print_numbered_item(num: usize, text: &str) {
    println!("  {}. {}", format!("{:>2}", num).dimmed(), text);
}

/// Print a simple list item with indent
pub fn print_list_item(text: &str) {
    println!("    {}", text);
}

/// Display full IDL overview
pub fn display_idl_overview(idl: &Idl) {
    print_header(&format!("Program: {}", idl.metadata.name));

    print_field("Version", &idl.metadata.version);
    print_field("Address", &idl.address);
    print_field("Spec", &idl.metadata.spec);

    if let Some(desc) = &idl.metadata.description {
        print_field("Description", desc);
    }

    print_subheader("Summary");
    println!(
        "  {} Instructions, {} Accounts, {} Types, {} Events, {} Errors",
        format!("{}", idl.instructions.len()).green(),
        format!("{}", idl.accounts.len()).yellow(),
        format!("{}", idl.types.len()).blue(),
        format!("{}", idl.events.len()).magenta(),
        format!("{}", idl.errors.len()).red(),
    );
    println!();
}

/// Display list of all instructions
pub fn display_instructions_list(idl: &Idl) {
    print_header(&format!(
        "Instructions for {} ({} total)",
        idl.metadata.name,
        idl.instructions.len()
    ));

    if idl.instructions.is_empty() {
        println!("  {}", "(none)".dimmed());
    } else {
        for (i, ix) in idl.instructions.iter().enumerate() {
            print_numbered_item(i + 1, &ix.name.green().to_string());
        }
    }
    println!();
}

/// Display detailed info for a single instruction
pub fn display_instruction_detail(instruction: &IdlInstruction) {
    print_header(&format!("Instruction: {}", instruction.name.green()));

    if !instruction.discriminator.is_empty() {
        print_field(
            "Discriminator",
            &format_discriminator(&instruction.discriminator),
        );
    }

    print_subheader(&format!(
        "Accounts ({})",
        count_accounts(&instruction.accounts)
    ));
    if instruction.accounts.is_empty() {
        println!("  {}", "(none)".dimmed());
    } else {
        display_account_items(&instruction.accounts, 1, 0);
    }

    print_subheader(&format!("Arguments ({})", instruction.args.len()));
    if instruction.args.is_empty() {
        println!("  {}", "(none)".dimmed());
    } else {
        for (i, arg) in instruction.args.iter().enumerate() {
            println!(
                "  {}. {} : {}",
                format!("{:>2}", i + 1).dimmed(),
                arg.name.yellow(),
                format_type(&arg.ty).blue()
            );
        }
    }
    println!();
}

/// Display account items (handles both Single and Group)
fn display_account_items(items: &[IdlAccountItem], start_num: usize, indent: usize) -> usize {
    let mut num = start_num;
    let indent_str = "  ".repeat(indent);

    for item in items {
        match item {
            IdlAccountItem::Single(account) => {
                let constraints = format_account_constraints(account);
                let extra = format_account_extra(account);

                println!(
                    "{}  {}. {} {}{}",
                    indent_str,
                    format!("{:>2}", num).dimmed(),
                    account.name.yellow(),
                    constraints,
                    extra
                );
                num += 1;
            }
            IdlAccountItem::Group(group) => {
                println!(
                    "{}  {} {}",
                    indent_str,
                    "▸".dimmed(),
                    group.name.white().bold()
                );
                num = display_account_items(&group.accounts, num, indent + 1);
            }
        }
    }
    num
}

/// Count total accounts (including nested)
fn count_accounts(items: &[IdlAccountItem]) -> usize {
    let mut count = 0;
    for item in items {
        match item {
            IdlAccountItem::Single(_) => count += 1,
            IdlAccountItem::Group(group) => count += count_accounts(&group.accounts),
        }
    }
    count
}

/// Format account constraints like [signer, writable]
fn format_account_constraints(account: &IdlAccount) -> String {
    let mut constraints = Vec::new();

    if account.signer {
        constraints.push("signer".green().to_string());
    }
    if account.writable {
        constraints.push("writable".magenta().to_string());
    }
    if account.optional {
        constraints.push("optional".dimmed().to_string());
    }

    if constraints.is_empty() {
        String::new()
    } else {
        format!("[{}]", constraints.join(", "))
    }
}

/// Format extra account info (address if present)
fn format_account_extra(account: &IdlAccount) -> String {
    if let Some(addr) = &account.address {
        format!(" {}", format!("({})", addr).dimmed())
    } else {
        String::new()
    }
}

/// Display list of all errors
pub fn display_errors_list(idl: &Idl) {
    print_header(&format!(
        "Errors for {} ({} total)",
        idl.metadata.name,
        idl.errors.len()
    ));

    if idl.errors.is_empty() {
        println!("  {}", "(none)".dimmed());
    } else {
        println!(
            "  {}  {}  {}",
            format!("{:<6}", "Code").dimmed(),
            format!("{:<24}", "Name").dimmed(),
            "Message".dimmed()
        );
        println!(
            "  {}  {}  {}",
            "─".repeat(6),
            "─".repeat(24),
            "─".repeat(30)
        );

        for error in &idl.errors {
            let msg = error.msg.as_deref().unwrap_or("-");
            println!(
                "  {}  {}  {}",
                format!("{:<6}", error.code).red(),
                format!("{:<24}", error.name).yellow(),
                msg.dimmed()
            );
        }
    }
    println!();
}

/// Format IdlType as readable string
pub fn format_type(ty: &IdlType) -> String {
    match ty {
        IdlType::Primitive(s) => s.clone(),
        IdlType::Complex(complex) => format_complex_type(complex),
    }
}

/// Format complex types
fn format_complex_type(ty: &IdlTypeComplex) -> String {
    match ty {
        IdlTypeComplex::Vec(inner) => format!("Vec<{}>", format_type(inner)),
        IdlTypeComplex::Option(inner) => format!("Option<{}>", format_type(inner)),
        IdlTypeComplex::Array(inner, size) => format!("[{}; {}]", format_type(inner), size),
        IdlTypeComplex::Defined { name } => name.clone(),
    }
}

/// Format discriminator bytes as hex
pub fn format_discriminator(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        "(none)".to_string()
    } else {
        let hex: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        format!("[{}]", hex.join(" "))
    }
}

/// Display an error message
pub fn display_error(msg: &str) {
    eprintln!("{} {}", "Error:".red().bold(), msg);
}

/// Display instruction not found error with suggestions
pub fn display_instruction_not_found(name: &str, available: &[&str]) {
    display_error(&format!("Instruction '{}' not found", name));

    if !available.is_empty() {
        eprintln!();
        eprintln!("{}", "Available instructions:".dimmed());
        for ix_name in available.iter().take(10) {
            eprintln!("  - {}", ix_name.green());
        }
        if available.len() > 10 {
            eprintln!(
                "  {} more...",
                format!("(+{})", available.len() - 10).dimmed()
            );
        }
    }
}
