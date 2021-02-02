#![allow(non_camel_case_types)]
#![allow(overflowing_literals)]
// #![no_std]
#![no_main]
#![feature(asm)]

use core::{ptr::null_mut, slice, usize};
use std::{ffi::OsString, str::FromStr};
use std::os::windows::prelude::*;
use std::ffi::CString;
use std::os::raw::c_char;
// use ntapi::winapi_local::um::winnt::__readgsqword;
// #[panic_handler]
// fn panic(_: &core::panic::PanicInfo) -> ! {
//     loop {}
// }
// https://stackoverflow.com/questions/48586816/converting-raw-pointer-to-16-bit-unicode-character-to-file-path-in-rust
unsafe fn u16_ptr_to_string(ptr: *const u16) -> OsString {
    let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(ptr, len);

    OsString::from_wide(slice)
}

unsafe fn u8_ptr_to_string(ptr: *const u8) -> String {
    // let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
    // let slice = std::slice::from_raw_parts(ptr, len);
    // String::from_utf8(slice).unwrap();
    // OsString::from_w
    let res = CString::from_raw(ptr as _);
    // let res = String::from_utf8_lossy(slice);
    return res.into_string().unwrap();

}

fn encode_wide_c(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::iter::once;

    OsStr::new(s).encode_wide().chain(once(0)).collect()
}
fn get_module_by_name(module_name: &str) ->PVOID {
    let module_name_bytes = encode_wide_c(module_name);
    dbg!(module_name_bytes.len());
    unsafe {
        let peb: *mut PEB;
        asm!(
            "mov {}, gs:[0x60]",
            out(reg) peb,
        );
        let ldr = (*peb).Ldr;
        let mut list = &((*ldr).InLoadOrderModuleList);
        let mut curr_module: *mut LDR_DATA_TABLE_ENTRY = &mut list as *mut _ as *mut _ ;
        println!("curr_module: {:p}", curr_module);
        println!("BaseAddress: {:p}", (*curr_module).BaseAddress);
        // let kernel32: &[u16] = wch_c!("kernel32.dll");
        let kernel32: &[u16] = &[107, 101, 114, 110, 101, 108, 51, 50, 46, 100, 108, 108, 0];
        loop {
            println!("start1...");
            // if curr_module
            if curr_module.is_null() || (*curr_module).BaseAddress.is_null() {
                println!("gg, {}, {}",curr_module.is_null() ,  (*curr_module).BaseAddress.is_null());
                // break;
            } 
            let mut curr_name = (*curr_module).BaseDllName.Buffer;
            if curr_name.is_null() {
                // continue;
            }
            // curr_name = curr_name.offset(1)
            println!("1");
            let mut i: isize = 0;
            println!("2");
            println!("curr_name: {:?}", curr_name);
            
            if curr_name.is_null() {

            } else {

                let name = u16_ptr_to_string(curr_name);
                println!("name===: {} {:?}", name.len(), name);
                if name.len() == module_name_bytes.len() -1 {
                    println!("base: {:?}", (*curr_module).BaseAddress);
                    // break;
                    return (*curr_module).BaseAddress;
                }
            }
            // for
                // break;
            let flink = (*curr_module).InLoadOrderModuleList.Flink;
            curr_module = flink as *mut LDR_DATA_TABLE_ENTRY ;
        }
        // println!("")
    }
}


fn get_func_by_name(module: PVOID, func_name: &str) {
    let idh: *const IMAGE_DOS_HEADER = module as *const _;
    unsafe {
        if (*idh).e_magic != IMAGE_DOS_SIGNATURE {
            println!("e_magic eror");
        } else {
            println!("magic ok");
        }
        let e_lfanew = (*idh).e_lfanew;
        dbg!(e_lfanew);
        let nt_headers: *const IMAGE_NT_HEADERS = (module as *const u8).offset(e_lfanew as isize) as *const _;
        println!("1");
        let op_header = &(*nt_headers).OptionalHeader;
        let exp_dir= &op_header.DataDirectory[0];

        let exp_addr = exp_dir.VirtualAddress;
        if exp_addr == 0 {
            println!("virtualaddr error");
        } else {
            println!("virtualAddr: 0x{:x} {}", exp_addr, exp_addr);
        }
        // let exp_dir_raw = exp_dir as *const _ as *const u8;
        println!("2");
        let exp: *const IMAGE_EXPORT_DIRECTORY = (module as *const u8).offset(exp_addr as _) as _; // this case error?
        println!("3");
        let names_count = (*exp).NumberOfNames;
        
        println!("4: {}", names_count);
        let funcs_rva = (*exp).AddressOfFunctions;
        println!("5");
        let func_names_rva = (*exp).AddressOfNames;
        println!("6");
        let names_ords_rva = (*exp).AddressOfNameOrdinals;

        println!("names_count: {}", names_count);
        for i in 0..names_count {
            println!("=== {} ===", i);
            let name_rva: *const DWORD = (module as *const u8).offset((func_names_rva + i*4) as isize) as *const _;
            let name_index: *const WORD = (module as *const u8).offset((names_ords_rva  + i*2) as isize)as *const _;
            let name_i = name_index.as_ref().unwrap();
            let mut off1:u32 =  (2 * name_i) as u32;
            off1 = off1 + funcs_rva;
            let func_rva: *const DWORD = (module as *const u8).offset(off1 as isize) as *const _;

            let mut rav_i = name_rva.as_ref().unwrap();
            let curr_name =  (module as *const u8).offset(*rav_i as isize);

            if !curr_name.is_null() {
                // let bla= CString::from_raw(curr_name as _);
            }
                let len = (0..).take_while(|&i| *curr_name.offset(i) != 0).count();
            let slice = std::slice::from_raw_parts(curr_name, len);
            // println!("cur_name: {:?}",slice);
            for i in slice {
                print!("{}", *i as char);
            }
            println!("");
            let c_string = CString::new("LoadLibraryA").expect("CString::new failed");
            let load_library = c_string.as_bytes();
            
            // for i in 0..load_library.len() {
            //     if load_library[i] == slice[i] {

            //     } else {
            //         println!("{} => {}", i, load_library.len());
            //         if load_library.len() - i < 2 {
            //             println!("got it: {:?}", slice);
            //         }
            //         break;
                    
            //     }
            // }
            
            // i


        }

    }
}

#[no_mangle]
// #[link_section = ".text.prologue"]
pub extern "C" fn main() -> ! {
    // __readgsqword(0x60);
    // KdPrint!("hello\n");
    let kk = get_module_by_name("KERNEL32.DLL");
    get_func_by_name(kk, "Vir");
    println!("kernel32: {:p}", kk);
    loop {}
}
/// NT Status type.
pub type NTSTATUS = Status;

/// A specialized `Result` type for NT operations.
pub type Result<T> = ::core::result::Result<T, Status>;


/// NT Status code.
#[repr(C)]
#[derive(Clone, Copy)]
pub enum Status {
	success = 0,
	unsuccessful = 0xC0000001,
}

extern "C"
{
	// /// `DbgPrint` routine sends a message to the kernel debugger.
	// pub fn DbgPrint(Format: *const u8, ...) -> NTSTATUS;
	// /// The `DbgPrintEx` routine sends a string to the kernel debugger if certain conditions are met.
	// pub fn DbgPrintEx(ComponentId: u32, Level: u32, Format: *const u8, ...) -> NTSTATUS;
}

// #[macro_export]
// macro_rules! KdPrint {
// 	($msg:expr $(, $arg:expr)*) => { unsafe { DbgPrint( concat!($msg, "\0").as_ptr() $(, $arg )* )} };
// }
pub enum c_void {}
type BOOLEAN = u8;
type HANDLE = *mut c_void;
type PVOID = *mut c_void;
type ULONG = u32;
type LPSTR = *mut i8;
#[repr(C)]
pub struct PEB {
  pub InheritedAddressSpace: BOOLEAN,
  pub ReadImageFileExecOptions: BOOLEAN,
  pub BeingDebugged: BOOLEAN,
  pub BitField: BOOLEAN,
  pub Mutant: HANDLE,
  pub ImageBaseAddress: PVOID,
  pub Ldr: *mut PEB_LDR_DATA,
  pub ProcessParameters: *mut RTL_USER_PROCESS_PARAMETERS,
}

#[repr(C)]
pub struct PEB_LDR_DATA {
  pub Length: ULONG,
  pub Initialized: BOOLEAN,
  pub SsHandle: HANDLE,
  pub InLoadOrderModuleList: LIST_ENTRY,
  // ...
}


#[repr(C)]
pub struct LDR_DATA_TABLE_ENTRY {
    pub InLoadOrderModuleList: LIST_ENTRY,
    pub InMemoryOrderModuleList: LIST_ENTRY,
    pub InInitializationOrderModuleList: LIST_ENTRY,
    pub BaseAddress: PVOID,
    pub EntryPoint: PVOID,
    pub SizeOfImage: ULONG,
    pub FullDllName: UNICODE_STRING,
    pub BaseDllName: UNICODE_STRING,

    // ...
}


type USHORT = u16;
type PWCH = *mut u16;
type DWORD = u32;
type WORD = u16;
type ULONGLONG = u64;
type BYTE = u8;
type LONG = u32;

#[repr(C)]
pub struct UNICODE_STRING {
    pub Length: USHORT,
    pub MaximumLength: USHORT,
    pub Buffer: PWCH,
}

#[repr(C)]
pub struct LIST_ENTRY {
    pub Flink: *mut LIST_ENTRY,
    pub Blink: *mut LIST_ENTRY,
}
#[repr(C)]
pub struct RTL_USER_PROCESS_PARAMETERS {
  pub MaximumLength: ULONG,
  pub Length: ULONG,
  pub Flags: ULONG,
  pub DebugFlags: ULONG,
  pub ConsoleHandle: HANDLE,
  pub ConsoleFlags: ULONG,
  pub StandardInput: HANDLE,
  pub StandardOutput: HANDLE,
  pub StandardError: HANDLE,
}


type PULONG = *mut ULONG;
#[repr(C)]
pub struct IO_STATUS_BLOCK {
  _1: IO_STATUS_BLOCK_u,
  _2: PULONG,
}

#[repr(C)]
pub union IO_STATUS_BLOCK_u {
  _1: NTSTATUS,
  _2: PVOID,
}


// ====
#[repr(C)]pub struct IMAGE_DATA_DIRECTORY {
    pub VirtualAddress: DWORD,
    pub Size: DWORD,
}
#[repr(C)]pub struct IMAGE_DOS_HEADER {
    pub e_magic: WORD,
    pub e_cblp: WORD,
    pub e_cp: WORD,
    pub e_crlc: WORD,
    pub e_cparhdr: WORD,
    pub e_minalloc: WORD,
    pub e_maxalloc: WORD,
    pub e_ss: WORD,
    pub e_sp: WORD,
    pub e_csum: WORD,
    pub e_ip: WORD,
    pub e_cs: WORD,
    pub e_lfarlc: WORD,
    pub e_ovno: WORD,
    pub e_res: [WORD; 4],
    pub e_oemid: WORD,
    pub e_oeminfo: WORD,
    pub e_res2: [WORD; 10],
    pub e_lfanew: LONG,
}

#[repr(C)]pub struct IMAGE_EXPORT_DIRECTORY {
    pub Characteristics: DWORD,
    pub TimeDateStamp: DWORD,
    pub MajorVersion: WORD,
    pub MinorVersion: WORD,
    pub Name: DWORD,
    pub Base: DWORD,
    pub NumberOfFunctions: DWORD,
    pub NumberOfNames: DWORD,
    pub AddressOfFunctions: DWORD,
    pub AddressOfNames: DWORD,
    pub AddressOfNameOrdinals: DWORD,
}
type ULONG_PTR = usize;

pub const IMAGE_DOS_SIGNATURE: WORD = 0x5A4D;

type IMAGE_NT_HEADERS = IMAGE_NT_HEADERS64;
#[repr(C)]pub struct IMAGE_NT_HEADERS64 {
    pub Signature: DWORD,
    pub FileHeader: IMAGE_FILE_HEADER,
    pub OptionalHeader: IMAGE_OPTIONAL_HEADER64,
}

#[repr(C)]pub struct IMAGE_FILE_HEADER {
    pub Machine: WORD,
    pub NumberOfSections: WORD,
    pub TimeDateStamp: DWORD,
    pub PointerToSymbolTable: DWORD,
    pub NumberOfSymbols: DWORD,
    pub SizeOfOptionalHeader: WORD,
    pub Characteristics: WORD,
}
#[repr(C)]pub struct IMAGE_OPTIONAL_HEADER64 {
    pub Magic: WORD,
    pub MajorLinkerVersion: BYTE,
    pub MinorLinkerVersion: BYTE,
    pub SizeOfCode: DWORD,
    pub SizeOfInitializedData: DWORD,
    pub SizeOfUninitializedData: DWORD,
    pub AddressOfEntryPoint: DWORD,
    pub BaseOfCode: DWORD,
    pub ImageBase: ULONGLONG,
    pub SectionAlignment: DWORD,
    pub FileAlignment: DWORD,
    pub MajorOperatingSystemVersion: WORD,
    pub MinorOperatingSystemVersion: WORD,
    pub MajorImageVersion: WORD,
    pub MinorImageVersion: WORD,
    pub MajorSubsystemVersion: WORD,
    pub MinorSubsystemVersion: WORD,
    pub Win32VersionValue: DWORD,
    pub SizeOfImage: DWORD,
    pub SizeOfHeaders: DWORD,
    pub CheckSum: DWORD,
    pub Subsystem: WORD,
    pub DllCharacteristics: WORD,
    pub SizeOfStackReserve: ULONGLONG,
    pub SizeOfStackCommit: ULONGLONG,
    pub SizeOfHeapReserve: ULONGLONG,
    pub SizeOfHeapCommit: ULONGLONG,
    pub LoaderFlags: DWORD,
    pub NumberOfRvaAndSizes: DWORD,
    pub DataDirectory: [IMAGE_DATA_DIRECTORY; 16],
}