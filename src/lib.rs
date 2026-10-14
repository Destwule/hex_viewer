use std::{
    fs, io::{self, Write, Read, Seek, SeekFrom}, path, cmp
};

#[derive(Default)]
pub struct HexDump {
    input_path: path::PathBuf,
    output_path: path::PathBuf,
    cols: u64,
    seek: u64,
    len: Option<u64>,
    use_uppercase: bool,
    use_autoskip: bool,

    prev_was_zero: bool,
    star_printed: bool,
}

impl HexDump {
    pub fn new() -> Self {
        let dump = Self::default();
        
        Self {
            cols: 16,
            ..dump
        }
    }

    fn get_input_path(&self) -> &path::PathBuf { &self.input_path }
    fn get_output_path(&self) -> &path::PathBuf { &self.output_path }
    fn get_cols(&self) -> u64 { self.cols }
    fn get_seek(&self) -> u64 { self.seek }
    fn get_len(&self) -> Option<u64> { self.len }
    fn use_uppercase(&self) -> bool { self.use_uppercase }
    fn use_autoskip(&self) -> bool { self.use_autoskip }
    fn prev_was_zero(&self) -> bool { self.prev_was_zero }
    fn star_printed(&self) -> bool { self.star_printed }
    
    pub fn set_cols(&mut self, new_cols: u64) { self.cols = new_cols; }
    pub fn set_seek(&mut self, new_seek: u64) { self.seek = new_seek; }
    pub fn set_len(&mut self, new_len: Option<u64>) { self.len = new_len; }
    pub fn set_uppercase(&mut self, value: bool) { self.use_uppercase = value; }
    pub fn set_autoskip(&mut self, value: bool) { self.use_autoskip = value; }
    pub fn set_input_path(&mut self, value: path::PathBuf) { self.input_path = value; }
    pub fn set_output_path(&mut self, value: path::PathBuf) { self.output_path = value; }
    
    fn set_prev_was_zero(&mut self, value: bool) { self.prev_was_zero = value; }
    fn set_star_printed(&mut self, value: bool) { self.star_printed = value; }
    
    fn has_filename(path: &path::PathBuf) -> bool {
        !path.is_empty()
    }

    fn open_output(&self) -> io::Result<Box<dyn Write>> {
        if Self::has_filename(self.get_output_path()) {
            return Ok(Box::new(fs::OpenOptions::new()
                .write(true).truncate(true).create(true)
                .open(self.get_output_path())?)
            );
        }

        Ok(Box::new(io::stdout().lock()))
    }

    fn open_input(&self) -> io::Result<fs::File> {
        Ok(fs::File::open(self.get_input_path())?)
    }

    pub fn display_contents(&mut self, handle: &mut Box<dyn Write>, offset_so_far: u64, contents: &[u8]) -> io::Result<()> {        
        let mut ascii_text = String::new();
        let mut padding = self.get_cols();
 
        if self.use_autoskip() {
            if contents.iter().all(|x| *x == 0) {
                if self.prev_was_zero() {
                    if !self.star_printed() {
                        writeln!(handle, "*")?;
                        self.set_star_printed(true);
                        return Ok(());
                    } else {
                        return Ok(());
                    }
                } else {
                    self.set_prev_was_zero(true);
                }
            } else {
                self.set_prev_was_zero(false);
                self.set_star_printed(false);
            }
        }
 
        write!(handle, "{:08x}:", self.get_seek() + offset_so_far)?;
 
        for byte in contents {
            if self.use_uppercase() {
                write!(handle, " {byte:02X}")?;
            } else {
                write!(handle, " {byte:02x}")?;
            }
 
            let display = if *byte >= 32 && *byte <= 126 {
                *byte as char
            } else { '.' };
            ascii_text.push(display);
        }
        padding += ((self.get_cols() - contents.len() as u64) * 2) + 2;
        writeln!(handle, "{ascii_text:>w$}", w = padding as usize)?;
        
        Ok(())
    }
    
    
    pub fn run(&mut self) -> io::Result<()> {
        if self.cols == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cols must be at least 1",
            ));
        }
        
        
        let mut handle = self.open_output()?;
        
        if self.get_input_path().is_empty() {
            use std::io::IsTerminal;
            let mut stdin = io::stdin().lock();
            let interactive = stdin.is_terminal();
            let cols = self.get_cols() as usize;
            let mut contents = vec![0u8; cols];
            
            loop {
                let mut buffer = Vec::new();
                let got = stdin.read_to_end(&mut buffer)?;
                
                if got == 0 {
                    if interactive { continue; }
                    std::process::exit(0);
                }
                
                let mut cursor = io::Cursor::new(buffer);
                cursor.seek(SeekFrom::Start(self.get_seek()))?;

                let mut total_read = 0u64;
                loop {
                    let remaining = match self.get_len() {
                        Some(limit) => limit.saturating_sub(total_read),
                        None => self.get_cols(),
                    };
                    let want = cmp::min(remaining, self.get_cols()) as usize;
                    if want == 0 { break; }
                    
                    let n = cursor.read(&mut contents[..want])?;
                    if n == 0 { break; }
                    
                    self.display_contents(&mut handle, total_read, &contents[..n])?;
                    total_read += n as u64;
                }
                
                handle.flush()?;
            }
        }
        
        
        if Self::has_filename(self.get_input_path()) && Self::has_filename(self.get_output_path()) {
            if let (Ok(a), Ok(b)) = (
                fs::canonicalize(&self.input_path),
                fs::canonicalize(&self.output_path),
            ) {
                if a == b {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "input and output are the same file",
                    ));
                }
            }
        }
        
        let mut input_file = self.open_input()?;
        let _ = input_file.seek(SeekFrom::Start(self.get_seek()));
        
        let mut contents: Vec<u8> = vec![0u8; self.get_cols() as usize];
        let mut total_read: u64 = 0;
        loop {
            let remaining: u64 = match self.get_len() {
                Some(limit) => limit.saturating_sub(total_read),
                None => self.get_cols(),
            };
            let want = cmp::min(remaining, self.get_cols());

            let n = input_file.read(&mut contents[..want as usize])?;
            if n == 0 {
                break;
            }
            
            self.display_contents(&mut handle, total_read, &contents[..n])?;
            total_read += n as u64;

            if let Some(limit) = self.get_len() {
                if total_read >= limit {
                    break;
                }
            }
        }

        Ok(())
    }
}

