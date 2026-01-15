import sys
import os
import shutil
import subprocess

# Base path for DLLs in the Fedora MinGW environment
DLL_SEARCH_PATH = "/usr/x86_64-w64-mingw32/sys-root/mingw/bin"

# Set of system DLLs to ignore (standard Windows DLLs)
SYSTEM_DLLS = {
    "advapi32.dll", "bcrypt.dll", "comctl32.dll", "comdlg32.dll",
    "crypt32.dll", "dbghelp.dll", "dwmapi.dll", "gdi32.dll",
    "gdiplus.dll", "imm32.dll", "iphlpapi.dll", "kernel32.dll",
    "msimg32.dll", "msvcrt.dll", "normaliz.dll", "ole32.dll",
    "oleaut32.dll", "opengl32.dll", "powrprof.dll", "psapi.dll",
    "rpcrt4.dll", "secur32.dll", "setupapi.dll", "shell32.dll",
    "shlwapi.dll", "ucrtbase.dll", "user32.dll", "userenv.dll",
    "usp10.dll", "uxtheme.dll", "version.dll", "winmm.dll",
    "winnls.dll", "ws2_32.dll", "wsock32.dll", "d3d9.dll", "dxgi.dll",
    "d3d11.dll", "dwrite.dll", "ncrypt.dll", "dnsapi.dll"
}

# Core MinGW libraries that are often required but might be missed by simple recursion
# if they are injected implicitly or used via specific mechanisms.
FORCE_INCLUDE = [
    "libstdc++-6.dll",
    "libwinpthread-1.dll",
    "libgcc_s_seh-1.dll", 
    "libtiff-5.dll", # Explicitly requested by user
    "libtiff-6.dll", # Possible alternative
]

def get_dependencies(file_path):
    deps = set()
    try:
        # Run objdump to get the import table
        output = subprocess.check_output(["x86_64-w64-mingw32-objdump", "-p", file_path]).decode("utf-8", errors="ignore")
        for line in output.splitlines():
            line = line.strip()
            if line.startswith("DLL Name:"):
                # Extract the DLL name
                dll_name = line.split("DLL Name:")[1].strip()
                deps.add(dll_name)
    except Exception as e:
        print(f"Error checking {file_path}: {e}")
    return deps

def find_dll_case_insensitive(dll_name, search_path):
    """Finds a file in search_path matching dll_name case-insensitively."""
    # First try direct match
    p = os.path.join(search_path, dll_name)
    if os.path.exists(p):
        return p
    
    # Try all lowercase
    p = os.path.join(search_path, dll_name.lower())
    if os.path.exists(p):
        return p
        
    # Slow fallback: list directory
    try:
        lower_name = dll_name.lower()
        for f in os.listdir(search_path):
            if f.lower() == lower_name:
                return os.path.join(search_path, f)
    except OSError:
        pass
        
    return None

def main(exe_path, output_dir):
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    processed = set()
    queue = [exe_path]
    
    # Add forced includes to the queue if they exist in the system
    for manual_dll in FORCE_INCLUDE:
        # verify existence before adding to queue to avoid "not found" warnings for alternatives
        found_path = find_dll_case_insensitive(manual_dll, DLL_SEARCH_PATH)
        if found_path:
             # We treat it as if found as a dependency
             print(f"Force including core library: {manual_dll}")
             queue.append(found_path) 
             # Note: logic below expects queue to contain paths or names for lookup?
             # My logic below handles queue items as 'names' mostly if coming from deps, but here I put a path.
             # Let's adjust logic to handle both or standardize.
             # EASIER: Just append to queue_deps inside the loop if we restructure?
             # No, let's just copy them directly here and add their deps.
             
             dst_path = os.path.join(output_dir, os.path.basename(found_path))
             if not os.path.exists(dst_path):
                 shutil.copy2(found_path, dst_path)
             
             # Add to processing set so we don't re-copy
             processed.add(os.path.basename(found_path).lower())
             processed.add(manual_dll.lower())
             
             # Add manual dll to queue to check ITS dependencies too
             # Use the just-copied path as the reference for dependency checking
             queue.append(dst_path)


    print(f"Gathering dependencies for {exe_path} into {output_dir}...")
    
    # Initial dependencies
    queue_deps = list(get_dependencies(exe_path))
    
    while queue_deps:
        dll_name = queue_deps.pop(0)
        dll_lower = dll_name.lower()
        
        if dll_lower in processed:
            continue
        
        if dll_lower in SYSTEM_DLLS or dll_lower.startswith("api-ms-win"):
            processed.add(dll_lower)
            continue
            
        processed.add(dll_lower)
        
        # Find the actual file
        src_path = find_dll_case_insensitive(dll_name, DLL_SEARCH_PATH)
            
        if src_path:
            dst_path = os.path.join(output_dir, os.path.basename(src_path))
            if not os.path.exists(dst_path):
                print(f"  Bundling {os.path.basename(src_path)}")
                shutil.copy2(src_path, dst_path)
                
                # Check dependencies of this DLL
                new_deps = get_dependencies(src_path)
                for new_dep in new_deps:
                    if new_dep.lower() not in processed:
                        queue_deps.append(new_dep)
        else:
            print(f"  Warning: Could not find {dll_name}")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python3 gather_deps.py <exe_path> <output_dir>")
        sys.exit(1)
    
    main(sys.argv[1], sys.argv[2])
