extern crate libc;

use self::libc::*;
use std::ffi::CString;
use std::ptr::null;

fn vec_to_c_str_ptr(command: &Vec<String>) -> (Vec<CString>, Vec<*const c_char>) {
    let mut owned_strings: Vec<CString> = Vec::with_capacity(command.len());
    let mut str_vec: Vec<*const c_char> = Vec::with_capacity(command.len() + 1);

    for ref word in command {
        let c_string = CString::new(word.as_str()).unwrap();
        let s_ptr = c_string.as_ptr();

        owned_strings.push(c_string);
        str_vec.push(s_ptr);
    }

    str_vec.push(null());
    (owned_strings, str_vec)
}

pub fn builtin_cd(command: &Vec<String>) {
    if command.len() <= 1 {
        eprintln!("cd requires an argument");
    } else {
        let dir = &command[1];
        unsafe {
            if chdir(CString::new(dir.as_str()).unwrap().as_ptr()) != 0 {
                eprintln!("Error running cd");
            }
        }
    }
}

pub fn run_builtin(command: &Vec<String>) {

}

pub fn run_file(command: &Vec<String>) {
    unsafe {
        let (_owned_strs, argv_vec) = vec_to_c_str_ptr(command);
        let argv: *const *const c_char = argv_vec.as_ptr();
        let cmd: *const c_char = *argv;

        let pid = fork();

        if pid == 0 {
            // child process
            if execvp(cmd, argv) < 0 {
                panic!("Fatal error: exec returned")
            }
        } else if pid < 0 {
            // error
            panic!("Fatar error with fork")
        } else {
            // parent process
            let mut status: c_int = 0;
            loop {
                waitpid(pid, &mut status as *mut c_int, WUNTRACED);
                if WIFEXITED(status) || WIFSIGNALED(status) { break; }
            }
        }
    }
}

pub fn run_command(command: &Vec<String>) {
    if command.is_empty() { return; }

    if command[0] == "cd" {
        builtin_cd(command);
    } else {
        run_file(command);
    }
}
