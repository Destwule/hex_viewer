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
        
        dump
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
    
    fn set_prevWasZero(&mut self, value: bool) { self.prev_was_zero = value; }
    fn set_starPrinted(&mut self, value: bool) { self.star_printed = value; }
    
    fn has_filename(path: &path::PathBuf) -> bool {
        !path.is_empty()
    }

    fn open_output(&self) -> io::Result<Box<dyn Write>> {
        if Self::has_filename(self.get_output_path()) {
            return Ok(Box::new(fs::OpenOptions::new()
                .write(true).create(true)
                .open(self.get_output_path())?)
            );
        }

        Ok(Box::new(io::stdout().lock()))
    }

    pub fn display_contents(&mut self, handle: &mut Box<dyn Write>, offset_so_far: u64, contents: &[u8]) -> io::Result<()> {        
        let mut ascii_text = String::new();
        let mut padding = self.get_cols();
 
        if self.use_autoskip() {
            if contents.iter().all(|x| *x == 0) {
                if self.prev_was_zero() {
                    if !self.star_printed() {
                        writeln!(handle, "*")?;
                        self.set_starPrinted(true);
                        return Ok(());
                    } else {
                        return Ok(());
                    }
                } else {
                    self.set_prevWasZero(true);
                }
            } else {
                self.set_prevWasZero(false);
                self.set_starPrinted(false);
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
        
        // manual flushing after done
        handle.flush()?;
        Ok(())
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut file = match fs::File::open(self.get_input_path()) {
            Ok(file) =>  file,
            Err(err) => return Err(io::Error::new(err.kind(), format!("Error opening input_file: {err}")))
        };
        let _ = file.seek(SeekFrom::Start(self.get_seek()));
        let mut handle = self.open_output()?;

        let mut contents: Vec<u8> = vec![0u8; self.get_cols() as usize];
        let mut total_read: u64 = 0;
        loop {
            let remaining: u64 = match self.get_len() {
                Some(limit) => limit.saturating_sub(total_read),
                None => self.get_cols(),
            };
            let want = cmp::min(remaining, self.get_cols());

            let n = file.read(&mut contents[..want as usize])?;
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

