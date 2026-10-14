use std::{
    path::{PathBuf}
};
use parse_clargs::{
    Parser, Arg, Value, ValueKind
};
use hex_viewer::HexDump;

fn main() -> Result<(), String> {
    let cli_args: Vec<String> = std::env::args().skip(1).collect();
    
    let cli_parser = Parser::new("hex_viewer").help_on_empty(false)
        .arg(
            Arg::positional("infile").long("infile").help("file to read from").value_kind(ValueKind::Str))?
        .arg(
            Arg::positional("outfile").long("outfile").help("file to write to").value_kind(ValueKind::Str))?
        .arg(
            Arg::new("cols").short("c").help("bytes per row (default 16)").value_kind(ValueKind::Num).default_value(Value::Num(16)))?
        .arg(
            Arg::new("len").long("len").short("l").help("Stop after N bytes").value_kind(ValueKind::Num))?
        .arg(
            
            Arg::new("seek").long("seek").short("s").help("start at byte offser N").value_kind(ValueKind::Num))?
        .arg(
            Arg::new("uppercase").long("uppercase").short("u").help("uppercase hex digits").value_kind(ValueKind::Bool).default_value(Value::Bool(false)))?
        .arg(
            Arg::new("autoskip").long("autoskip").short("a").help("collapse repeated all-zero rows into '*'").value_kind(ValueKind::Bool).default_value(Value::Bool(false)))?;
    
    let matches = cli_parser.parse(&cli_args)?;
    let mut hex_dump = HexDump::new();

    let infile = matches.get_str("infile").unwrap_or("");
    let outfile = matches.get_str("outfile").unwrap_or("");
    let cols = matches.get_num("cols").unwrap();
    let seek = matches.get_num("seek").unwrap_or(0);
    let len = matches.get_num("len");
    let uppercase = matches.get_bool("uppercase").unwrap();
    let autoskip = matches.get_bool("autoskip").unwrap();

    hex_dump.set_seek(seek);
    hex_dump.set_cols(cols);
    hex_dump.set_len(len);
    hex_dump.set_autoskip(autoskip);
    hex_dump.set_uppercase(uppercase);
    hex_dump.set_len(len);
    hex_dump.set_input_path(PathBuf::from(infile));
    hex_dump.set_output_path(PathBuf::from(outfile));

    hex_dump.run().map_err(|err| format!("{err}"))?;

    Ok(())
}
