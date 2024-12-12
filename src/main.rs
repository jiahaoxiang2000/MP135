// filepath: /Users/xiangjiahao/embed/MP135/src/main.rs
use libc::{ioctl, open};
use std::ffi::CString;
use std::os::fd::AsRawFd;
use std::process::exit;


use libc::{
    close, lseek, mmap, munmap, PROT_WRITE, MAP_SHARED, O_RDWR, SEEK_SET,

};
use std::fs::File;
use std::io::{self, Read};
use std::mem::zeroed;
use std::ptr::null_mut;

#[repr(C, packed)]
struct BmpInfoHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bpp: u16,
    compression: u32,
    image_size: u32,
    x_pels_per_meter: i32,
    y_pels_per_meter: i32,
    clr_used: u32,
    clr_important: u32,
}

static mut FB_FIX: fb_fix_screeninfo = unsafe { zeroed() };
static mut FB_VAR: fb_var_screeninfo = unsafe { zeroed() };
static mut SCREEN_BASE: *mut u32 = null_mut();

#[repr(C, packed)]
struct BmpFileHeader {
    type_field: [u8; 2],
    size: u32,
    reserved1: u16,
    reserved2: u16,
    offset: u32,
}

fn read_bmp_header(file: &mut File) -> io::Result<BmpFileHeader> {
    let mut file_h = BmpFileHeader {
        type_field: [0; 2],
        size: 0,
        reserved1: 0,
        reserved2: 0,
        offset: 0,
    };

    // Read type_field (2 bytes)
    file.read_exact(&mut file_h.type_field)?;

    // Read size (4 bytes)
    let mut size_bytes = [0u8; 4];
    file.read_exact(&mut size_bytes)?;
    file_h.size = u32::from_le_bytes(size_bytes);

    // Read reserved1 (2 bytes)
    let mut reserved1_bytes = [0u8; 2];
    file.read_exact(&mut reserved1_bytes)?;
    file_h.reserved1 = u16::from_le_bytes(reserved1_bytes);

    // Read reserved2 (2 bytes)
    let mut reserved2_bytes = [0u8; 2];
    file.read_exact(&mut reserved2_bytes)?;
    file_h.reserved2 = u16::from_le_bytes(reserved2_bytes);

    // Read offset (4 bytes)
    let mut offset_bytes = [0u8; 4];
    file.read_exact(&mut offset_bytes)?;
    file_h.offset = u32::from_le_bytes(offset_bytes);

    Ok(file_h)
}

fn show_bmp_image(path: &str) -> i32 {
    unsafe {

        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(_) => {
                eprintln!("Failed to open BMP file");
                return -1;
            }
        };

        let mut file_h: BmpFileHeader = zeroed();
        if file.read_exact(&mut file_h.type_field).is_err()
            || file.read_exact(&mut file_h.size.to_le_bytes()).is_err()
            || file.read_exact(&mut file_h.reserved1.to_le_bytes()).is_err()
            || file.read_exact(&mut file_h.reserved2.to_le_bytes()).is_err()
            || file.read_exact(&mut file_h.offset.to_le_bytes()).is_err()
        {
            eprintln!("read error");
            return -1;
        }
        // print the file header all items on one line
        let type_field = file_h.type_field;
        let size = file_h.size;
        let reserved1 = file_h.reserved1;
        let reserved2 = file_h.reserved2;
        let offset = file_h.offset;
        print!("type_field: {:?}, size: {}, reserved1: {}, reserved2: {}, offset: {}\n", type_field, size, reserved1, reserved2, offset);

        if &file_h.type_field != b"BM" {
            eprintln!("it's not a BMP file");
            return -1;
        }

        let mut info_h: BmpInfoHeader = zeroed();
        // Read remaining bmp_info_header fields
        if let Err(e) = read_bmp_header(&mut file, &mut file_h) {
            eprintln!("Failed to read BMP header: {}", e);
            return -1;
        }
        let size = info_h.size; 
        let width = info_h.width;
        let height = info_h.height;
        print!("size: {}\n", size);
        print!("width: {}, height: {}\n", width, height);

        if lseek(file.as_raw_fd(), file_h.offset as i32, SEEK_SET) == -1 {
            eprintln!("lseek error");
            return -1;
        }

        let line_bytes = (info_h.width as i32 * (info_h.bpp as i32 / 8)) as usize;
        let mut line_buf = vec![0u8; line_bytes];
        let min_bytes = if (FB_FIX.line_length as usize) > line_bytes {
            line_bytes
        } else {
            FB_FIX.line_length as usize
        };

        let min_h;
        let mut base = SCREEN_BASE;
        let bytes_per_pixel = (FB_VAR.bits_per_pixel / 8) as usize;
        let width = (FB_FIX.line_length as usize / bytes_per_pixel) as usize;

        if info_h.height > 0 {
            min_h = if info_h.height > FB_VAR.yres as i32 {
                FB_VAR.yres as i32
            } else {
                info_h.height
            };
            // Position to the bottom-left if height is positive
            base = base.add(width * (min_h - 1) as usize);
            for _ in 0..min_h {
                if file.read_exact(&mut line_buf).is_err() {
                    eprintln!("read error");
                    return -1;
                }
                std::ptr::copy_nonoverlapping(
                    line_buf.as_ptr() as *mut u8,
                    base as *mut u8,
                    min_bytes,
                );
                base = base.sub(width);
            }
        } else {
            min_h = (-info_h.height) as i32;
            for _ in 0..min_h {
                if file.read_exact(&mut line_buf).is_err() {
                    eprintln!("read error");
                    return -1;
                }
                std::ptr::copy_nonoverlapping(
                    line_buf.as_ptr(),
                    base as *mut u8,
                    min_bytes,
                );
                base = base.add(width);
            }
        }

    }
    0
}





// Include the generated bindings.
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    if std::env::args().len() != 2 {
        eprintln!("usage: {} <bmp_file>", std::env::args().next().unwrap());
        exit(-1);
    }

    unsafe {
        let fd = open(CString::new("/dev/fb0").unwrap().as_ptr(), O_RDWR);
        if fd < 0 {
            eprintln!("open error");
            exit(-1);
        }

        if ioctl(fd, FBIOGET_VSCREENINFO, &mut FB_VAR as *mut _) < 0 {
            eprintln!("FBIOGET_VSCREENINFO error");
            close(fd);
            exit(-1);
        }

        if ioctl(fd, FBIOGET_FSCREENINFO, &mut FB_FIX as *mut _) < 0 {
            eprintln!("FBIOGET_FSCREENINFO error");
            close(fd);
            exit(-1);
        }

        let screen_size = FB_FIX.line_length * FB_VAR.yres;
        SCREEN_BASE = mmap(
            null_mut(),
            screen_size as usize,
            PROT_WRITE,
            MAP_SHARED,
            fd,
            0,
        ) as *mut u32;

        if SCREEN_BASE as *mut libc::c_void == libc::MAP_FAILED {
            eprintln!("mmap error");
            close(fd);
            exit(-1);
        }

        // Clear the screen
        for i in 0..(screen_size as usize / 4) {
            *SCREEN_BASE.add(i) = 0x00000000;
        }

        let bmp_path = std::env::args().nth(1).unwrap();
        let result = show_bmp_image(&bmp_path);
        if result != 0 {
            eprintln!("Failed to display BMP image");
        }
    
        let screen_size = FB_FIX.line_length * FB_VAR.yres;
        munmap(SCREEN_BASE as *mut _, screen_size as usize);

        close(fd);
    }

   

    exit(0);
}